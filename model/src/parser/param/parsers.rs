use super::helpers::{
    extract_string_enum_value, parse_enum_lines, strip_wiki_markup, try_optional,
};
use crate::{Call, NumberEnumValue, Param, ParamItem, StringEnumValue, Value};

/// Try parsing a simple line parameter
/// name: [[Type]] - Description
pub fn try_simple_line(source: &str) -> Result<Option<ParamItem>, String> {
    if source.contains('\n') {
        return Ok(None);
    }
    let (since, source) = if source.starts_with("{{") {
        super::super::extract_since(source)?
    } else {
        (None, source)
    };
    let Some((name_part, type_and_description)) = source.split_once(": ") else {
        return Ok(None);
    };

    // Try to split on " - " for description
    let (type_part, desc) =
        if let Some((type_part, description_part)) = type_and_description.split_once(" - ") {
            (type_part, Some(description_part.trim().to_string()))
        } else {
            // If no dash, try to extract type from beginning (things in [[ ]])
            // and treat the rest as description
            let trimmed = type_and_description.trim();
            trimmed.find("]]").map_or((trimmed, None), |end_bracket| {
                let potential_type = &trimmed[..end_bracket + 2];
                let potential_desc = trimmed[end_bracket + 2..].trim();

                // Verify this is a valid type by trying to parse it
                if Value::parse(potential_type, 0).is_ok() && !potential_desc.is_empty() {
                    (potential_type, Some(potential_desc.to_string()))
                } else {
                    (trimmed, None)
                }
            })
        };
    let typ = Value::parse(type_part.trim(), 0)?;
    let name = name_part.trim().to_string();
    let (default, optional, desc) = desc.map_or((None, false, None), |desc| {
        if let Some((default, desc)) = try_optional(&desc) {
            (default, true, Some(desc))
        } else {
            (None, false, Some(desc))
        }
    });
    Ok(Some(ParamItem {
        name,
        typ,
        desc,
        default,
        optional,
        since,
    }))
}

pub fn try_array_with(source: &str) -> Result<Option<ParamItem>, String> {
    if !source.contains('\n') {
        return Ok(None);
    }

    let Some((name_part, type_and_description)) = source.split_once(": ") else {
        return Ok(None);
    };

    let mut lines = type_and_description.lines();
    let first_line = lines.next().expect("first line").trim();
    let (first_line, wrap_arrays) = if first_line.starts_with("[[Array]] of ") {
        let first_line = first_line
            .trim_start_matches("[[Array]] of ")
            .trim()
            .replace("[[Array]]s with ", "[[Array]] with ");
        (first_line, true)
    } else {
        (first_line.to_string(), false)
    };
    if !first_line.starts_with("[[Array]] with ") {
        return Ok(None);
    }
    let (args, desc) = if first_line.contains(" - ") {
        let Some((params_part, description_part)) = first_line.split_once(" - ") else {
            return Err(format!("Invalid array with line: '{first_line}'"));
        };
        (
            params_part.trim_start_matches("[[Array]] with").trim(),
            Some(description_part.trim().to_string()),
        )
    } else {
        (first_line.trim_start_matches("[[Array]] with").trim(), None)
    };
    let Some(arg) = Call::parse_params(args) else {
        return Err(format!("Failed to parse array with parameters: '{args}'"));
    };
    let mut params = Vec::new();
    let mut in_columns = false;
    let lines_vec: Vec<&str> = lines.collect();
    let mut i = 0;

    while i < lines_vec.len() {
        let line = lines_vec[i].trim();

        // Handle {{Columns|...| opening
        if line.contains("{{Columns|") {
            in_columns = true;
            i += 1;
            continue;
        }

        // Handle }} closing
        if line == "}}" {
            if !in_columns {
                return Err("Unexpected closing '}}' without matching '{{Columns|'".to_string());
            }
            in_columns = false;
            i += 1;
            continue;
        }

        // Parse parameter lines (starting with *)
        if line.starts_with('*') && !line.starts_with("**") {
            let line_stripped = line.trim_start_matches('*').trim();

            // Check if this is a nested array_with pattern
            if line_stripped.contains("[[Array]] with") {
                // Collect this line and all following ** lines
                let mut nested_source = line_stripped.to_string();
                let mut nested_i = i + 1;

                while nested_i < lines_vec.len() {
                    let next_line = lines_vec[nested_i].trim();
                    if next_line.starts_with("**") {
                        nested_source.push('\n');
                        // Convert ** to * for nested parsing
                        nested_source.push('*');
                        nested_source.push_str(next_line.trim_start_matches("**").trim());
                        nested_i += 1;
                    } else {
                        break;
                    }
                }

                i = nested_i;

                // Try to parse as array_with
                if let Ok(Some(item)) = try_array_with(&nested_source) {
                    params.push(item);
                } else {
                    return Err(format!(
                        "Failed to parse nested array with element line: '{line_stripped}'"
                    ));
                }
            } else {
                // Regular simple line
                let line_str = line_stripped.to_string();
                // detect index, eg: 0 - {name}: [[Type]] - Description
                let line_str = if let Some((index, rest)) = line_str.split_once(" - ") {
                    if index.trim().chars().all(|c| c.is_ascii_digit()) {
                        rest.trim().to_string()
                    } else {
                        line_str
                    }
                } else {
                    line_str
                };
                if let Ok(Some(item)) = try_simple_line(&line_str) {
                    params.push(item);
                } else {
                    return Err(format!(
                        "Failed to parse array with element line: '{line_str}'"
                    ));
                }
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    let (default, optional, desc) = desc.map_or((None, false, None), |desc| {
        if let Some((default, desc)) = try_optional(&desc) {
            (default, true, Some(desc))
        } else {
            (None, false, Some(desc))
        }
    });
    let param = Param::build_from_arg(&arg, &params)?;
    Ok(Some(ParamItem {
        name: name_part.trim().to_string(),
        typ: if wrap_arrays {
            Value::ArrayUnsized {
                value: Box::new(param.as_value()),
            }
        } else {
            param.as_value()
        },
        desc,
        default,
        optional,
        since: None,
    }))
}

/// Try parsing a number enum parameter
/// Format: name: [[Number]] - description (optional : at end)
/// * 0 - enum value 0
/// * 1 - enum value 1
pub fn try_number_enum(source: &str) -> Result<Option<ParamItem>, String> {
    if !source.contains('\n') {
        return Ok(None);
    }

    let Some((name_part, type_and_description)) = source.split_once(": ") else {
        return Ok(None);
    };

    let mut lines = type_and_description.lines();
    let first_line = lines.next().unwrap_or("").trim();

    // Check if it's a number enum format: [[Number]] - description:
    if !first_line.starts_with("[[Number]]") {
        return Ok(None);
    }

    // Split on " - " to get description
    if !first_line.contains(" - ") {
        return Ok(None);
    }

    let (_, desc_part) = first_line.split_once(" - ").unwrap();

    let desc = desc_part.trim_end_matches(':').trim().to_string();

    // Parse enum values from remaining lines (skip empty lines)
    let mut enum_values = Vec::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('*') && !line.starts_with("**") {
            let line_content = line.trim_start_matches('*').trim();

            // Format: number - description
            if let Some((num_str, enum_desc)) = line_content.split_once(" - ")
                && let Ok(value) = num_str.trim().parse::<i32>()
            {
                let desc_text = strip_wiki_markup(enum_desc.trim());
                enum_values.push(NumberEnumValue {
                    value,
                    desc: Some(desc_text),
                    since: None,
                });
            }
        }
    }

    if enum_values.is_empty() {
        return Ok(None);
    }

    Ok(Some(ParamItem {
        name: name_part.trim().to_string(),
        typ: Value::NumberEnum(enum_values),
        desc: Some(desc),
        default: None,
        optional: false,
        since: None,
    }))
}

/// Try parsing a string enum parameter
/// Format: name: [[String]] - description, can be one of:
/// * {{hl|"VALUE"}}
/// * {{GVI|arma3|VERSION|...}} {{hl|"VALUE"}}
pub fn try_string_enum(source: &str) -> Result<Option<ParamItem>, String> {
    if !source.contains('\n') {
        return Ok(None);
    }

    let Some((name_part, type_and_description)) = source.split_once(": ") else {
        return Ok(None);
    };

    let mut lines = type_and_description.lines();
    let first_line = lines.next().unwrap_or("").trim();

    // Check if it's a string enum format: [[String]] - description
    if !first_line.starts_with("[[String]]") {
        return Ok(None);
    }

    // Split on " - " to get description
    if !first_line.contains(" - ") {
        return Ok(None);
    }

    let (_, desc_part) = first_line.split_once(" - ").unwrap();

    // Should contain "can be one of:" or similar enum indicator
    let desc_full = desc_part.trim_end_matches(':').trim();
    if !desc_full.to_lowercase().contains("can be one of") {
        return Ok(None);
    }

    // Extract just the description before "can be one of"
    let desc = if let Some(pos) = desc_full.to_lowercase().find("can be one of") {
        desc_full[..pos]
            .trim()
            .trim_end_matches(',')
            .trim()
            .to_string()
    } else {
        desc_full.to_string()
    };

    // Extract enum values using the helper function
    let enum_values: Vec<StringEnumValue> = parse_enum_lines(lines, |line_content| {
        extract_string_enum_value(line_content)
    })
    .into_iter()
    .map(|(value, since)| StringEnumValue {
        value,
        desc: None,
        since,
    })
    .collect();

    if enum_values.is_empty() {
        return Ok(None);
    }

    Ok(Some(ParamItem {
        name: name_part.trim().to_string(),
        typ: Value::StringEnum(enum_values),
        desc: Some(desc),
        default: None,
        optional: false,
        since: None,
    }))
}

/// Try parsing a simple line with extra description lines
pub fn try_simple_line_with_extra_description(source: &str) -> Result<Option<ParamItem>, String> {
    if !source.contains('\n') {
        return Ok(None);
    }

    let mut lines = source.lines();
    let first_line_raw = match lines.next() {
        Some(line) => line,
        None => return Ok(None),
    };

    let first_line = first_line_raw.trim();

    // Handle empty first lines
    if first_line.is_empty() {
        let first_line = match lines.next() {
            Some(line) => line.trim(),
            None => return Ok(None),
        };
        // Recursive call with non-empty first line
        let remaining = format!("{}\n{}", first_line, lines.collect::<Vec<_>>().join("\n"));
        return try_simple_line_with_extra_description(&remaining);
    }

    // Try to parse the first line as a simple parameter line
    let mut param_item = match try_simple_line(first_line)? {
        Some(item) => item,
        None => return Ok(None),
    };

    // Collect extra description lines (those starting with *)
    let mut extra_desc = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('*') {
            extra_desc.push(trimmed.to_string());
        }
    }

    // If we found extra description lines, append them to the description
    if !extra_desc.is_empty() {
        let extra_text = extra_desc.join("\n");
        if let Some(ref mut desc) = param_item.desc {
            desc.push('\n');
            desc.push_str(&extra_text);
        } else {
            param_item.desc = Some(extra_text);
        }
    }

    Ok(Some(param_item))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_simple_line() {
        let line = "speed: [[Number]] - The speed of the vehicle.";
        let result = try_simple_line(line).expect("Failed to parse simple line");
        let param_item = result.expect("Expected Some");
        assert_eq!(param_item.name, "speed");
        assert_eq!(
            param_item.desc.as_deref(),
            Some("The speed of the vehicle.")
        );
    }

    #[test]
    fn test_simple_line_with_range() {
        let line = "x: [[Number]] in range -1..+1 - any other value returns [[NaN]]";
        let result = try_simple_line(line).expect("Failed to parse simple line with range");
        let param_item = result.expect("Expected Some");
        assert_eq!(param_item.name, "x");
        assert_eq!(
            param_item.desc.as_deref(),
            Some("any other value returns [[NaN]]")
        );
    }

    #[test]
    fn test_number_enum() {
        let line = "return: [[Number]] - admin state of given client:
* 0 - client is not an admin
* 1 - client is admin that is [[Arma 3: Server Config File#Voted_In_Admin|voted in]]
* 2 - client is admin that is [[Arma 3: Server Config File#Logged_In_Admin|logged in]]";
        let result = try_number_enum(line).expect("Failed to parse number enum");
        let param_item = result.expect("Expected Some");
        assert_eq!(param_item.name, "return");
        assert_eq!(
            param_item.desc.as_deref(),
            Some("admin state of given client")
        );
        assert_eq!(
            param_item.typ,
            Value::NumberEnum(vec![
                NumberEnumValue {
                    value: 0,
                    desc: Some("client is not an admin".to_string()),
                    since: None,
                },
                NumberEnumValue {
                    value: 1,
                    desc: Some("client is admin that is voted in".to_string()),
                    since: None,
                },
                NumberEnumValue {
                    value: 2,
                    desc: Some("client is admin that is logged in".to_string()),
                    since: None,
                },
            ])
        );
    }

    #[test]
    fn test_string_enum() {
        let line = r#"shape: [[String]] - the shape, can be one of:
* {{hl|"ICON"}}
* {{hl|"RECTANGLE"}}
* {{hl|"ELLIPSE"}}
* {{GVI|arma3|1.60|size= 0.75}} {{hl|"POLYLINE"}}"#;
        let result = try_string_enum(line).expect("Failed to parse string enum");
        let param_item = result.expect("Expected Some");
        assert_eq!(param_item.name, "shape");
        assert_eq!(param_item.desc, Some("the shape".to_string()));
        assert_eq!(
            param_item.typ,
            Value::StringEnum(vec![
                StringEnumValue {
                    value: "ICON".to_string(),
                    desc: None,
                    since: None,
                },
                StringEnumValue {
                    value: "RECTANGLE".to_string(),
                    desc: None,
                    since: None,
                },
                StringEnumValue {
                    value: "ELLIPSE".to_string(),
                    desc: None,
                    since: None,
                },
                StringEnumValue {
                    value: "POLYLINE".to_string(),
                    desc: None,
                    since: Some(crate::Since::arma3("1.60")),
                },
            ])
        );
    }
}
