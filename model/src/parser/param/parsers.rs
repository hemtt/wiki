use super::helpers::{extract_string_enum_value_with_desc, strip_wiki_markup, try_optional};
use crate::{Call, NumberEnumValue, OneOfValue, Param, ParamItem, StringEnumValue, Value};

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

    let lines: Vec<&str> = type_and_description.lines().collect();
    if lines.is_empty() {
        return Ok(None);
    }

    let first_line = lines[0].trim();

    // Check if it's a string enum format: [[String]] - description
    if !first_line.starts_with("[[String]]") {
        return Ok(None);
    }

    // Split on " - " to get description
    if !first_line.contains(" - ") {
        return Ok(None);
    }

    let (_, desc_part) = first_line.split_once(" - ").unwrap();

    // Should contain "can be one of" or "- one of" or similar enum indicator
    let desc_full = desc_part.trim_end_matches(':').trim();
    let desc_lower = desc_full.to_lowercase();
    let has_enum_indicator = desc_lower.contains("can be one of")
        || desc_lower.contains("- one of")
        || desc_lower.contains("one of:");
    if !has_enum_indicator {
        return Ok(None);
    }

    // Extract just the description before the enum indicator
    let desc = if let Some(pos) = desc_lower.find("can be one of") {
        desc_full[..pos]
            .trim()
            .trim_end_matches(',')
            .trim()
            .to_string()
    } else if let Some(pos) = desc_lower.find("- one of") {
        desc_full[..pos]
            .trim()
            .trim_end_matches(',')
            .trim()
            .to_string()
    } else if let Some(pos) = desc_lower.find("one of:") {
        desc_full[..pos]
            .trim()
            .trim_end_matches(',')
            .trim()
            .to_string()
    } else {
        desc_full.to_string()
    };

    // Collect enum values with descriptions, handling {{Columns|...}} wrapper
    let mut enum_values = Vec::new();
    let mut in_columns = false;

    for i in 1..lines.len() {
        let line = lines[i].trim();

        // Handle {{Columns|...| opening
        if line.contains("{{Columns|") {
            in_columns = true;
            continue;
        }

        // Handle }} closing
        if line == "}}" {
            if in_columns {
                in_columns = false;
            }
            continue;
        }

        // Parse enum items (lines starting with *)
        if line.starts_with('*') && !line.starts_with("**") {
            let line_content = line.trim_start_matches('*').trim();

            // Extract version info if present (e.g., {{GVI|arma3|2.20|...}})
            let (since, content_for_value) = if line_content.starts_with("{{GVI|") {
                match super::super::extract_since(line_content) {
                    Ok((since, rest)) => (since, rest.trim()),
                    Err(_) => (None, line_content),
                }
            } else {
                (None, line_content)
            };

            // Try to extract the value and optional description
            if let Some((value, desc)) = extract_string_enum_value_with_desc(content_for_value) {
                enum_values.push(StringEnumValue { value, desc, since });
            }
        }
    }

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

/// Try parsing a multiple type enum parameter (OneOf with enums)
/// Format: name: [[Type1]] or [[Type2]] - description:
/// * value1 / value2 - description
pub fn try_multiple_type_enum(source: &str) -> Result<Option<ParamItem>, String> {
    if !source.contains('\n') {
        return Ok(None);
    }

    let Some((name_part, type_and_description)) = source.split_once(": ") else {
        return Ok(None);
    };

    let lines = type_and_description.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return Ok(None);
    }

    let first_line = lines[0].trim();

    // Check if it contains "or" for OneOf types
    if !first_line.contains(" or ") {
        return Ok(None);
    }

    // Parse types - e.g., [[Number]] or [[String]]
    if !first_line.contains(" - ") {
        return Ok(None);
    }

    let (types_part, desc_part) = first_line.split_once(" - ").unwrap();

    // Extract the description
    let desc = desc_part.trim_end_matches(':').trim().to_string();

    // Parse types to determine what we're dealing with
    let type_strings: Vec<&str> = types_part.split(" or ").map(|s| s.trim()).collect();
    if type_strings.len() != 2 {
        return Ok(None);
    }

    let type1_str = type_strings[0];
    let type2_str = type_strings[1];

    // Parse the first type - may have version info like {{GVI|arma3|2.04|...}} [[Number]]
    let (type1_since, type1_clean) = if type1_str.starts_with("{{") {
        match super::super::extract_since(type1_str) {
            Ok((since, rest)) => (since, rest.trim()),
            Err(_) => (None, type1_str),
        }
    } else {
        (None, type1_str)
    };

    let (type2_since, type2_clean) = if type2_str.starts_with("{{") {
        match super::super::extract_since(type2_str) {
            Ok((since, rest)) => (since, rest.trim()),
            Err(_) => (None, type2_str),
        }
    } else {
        (None, type2_str)
    };

    // Try to parse the types
    let type1_val = Value::parse(type1_clean, 0)?;
    let _type2_val = Value::parse(type2_clean, 0)?;

    // Collect entries from remaining lines
    let mut entries: Vec<(Option<String>, Option<String>, Option<String>)> = Vec::new(); // (val1, val2, description)
    let mut raw_line_contents: Vec<String> = Vec::new();

    for i in 1..lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('*') && !line.starts_with("**") {
            let line_content = line.trim_start_matches('*').trim();
            raw_line_contents.push(line_content.to_string());

            // Extract version info if present
            let (_, content_for_value) = if line_content.starts_with("{{GVI|") {
                match super::super::extract_since(line_content) {
                    Ok((since, rest)) => (since, rest.trim()),
                    Err(_) => (None, line_content),
                }
            } else {
                (None, line_content)
            };

            // Split on " - " or " = " to separate values from description
            let (values_part, desc_part) = if let Some(pos) = content_for_value.find(" - ") {
                (
                    &content_for_value[..pos],
                    Some(&content_for_value[pos + 3..]),
                )
            } else if let Some(pos) = content_for_value.find(" = ") {
                (
                    &content_for_value[..pos],
                    Some(&content_for_value[pos + 3..]),
                )
            } else {
                (content_for_value, None)
            };

            let desc = desc_part.map(|d| d.trim().to_string());

            // Try to split on "/" to get both values
            if let Some(idx) = values_part.find('/') {
                let value1 = values_part[..idx].trim().to_string();
                let value2 = values_part[idx + 1..].trim().to_string();
                entries.push((Some(value1), Some(value2), desc));
            } else {
                // No separator, just one value (could be for plain types or cases without pairs)
                entries.push((Some(values_part.trim().to_string()), None, desc));
            }
        }
    }

    if entries.is_empty() {
        return Ok(None);
    }

    // Determine how to create the OneOf based on type combinations
    let mut one_of_values: Vec<OneOfValue> = Vec::new();

    // Check if type1 is a plain type (like Boolean, String, etc.)
    // vs an enumerable type (Number can be both depending on context)
    // We treat Boolean, String, Code, Object, etc. as plain types
    let type1_is_plain = matches!(
        type1_val,
        Value::Boolean
            | Value::String
            | Value::Code
            | Value::Object
            | Value::Config
            | Value::Control
            | Value::Display
            | Value::Location
            | Value::Side
            | Value::Group
            | Value::DiaryRecord
    );

    if type1_is_plain {
        // First type is plain (e.g., Boolean), attach all raw lines as description
        let all_desc = raw_line_contents.join("\n");
        one_of_values.push(OneOfValue {
            typ: type1_val,
            desc: Some(all_desc),
            since: type1_since,
        });

        // Parse second type as enum (extract first value for each line as enum value)
        let mut enum_values: Vec<NumberEnumValue> = Vec::new();
        for (val1, _, desc) in &entries {
            if let Some(val_str) = val1 {
                if let Ok(num) = val_str.parse::<i32>() {
                    enum_values.push(NumberEnumValue {
                        value: num,
                        desc: desc.clone(),
                        since: None,
                    });
                }
            }
        }

        if !enum_values.is_empty() {
            one_of_values.push(OneOfValue {
                typ: Value::NumberEnum(enum_values),
                desc: None,
                since: type2_since,
            });
        }
    } else {
        // Both types can have enums - create enums for both

        // Parse first type as enum (numeric)
        let mut enum1_values: Vec<NumberEnumValue> = Vec::new();
        for (val1, _, desc) in &entries {
            if let Some(val_str) = val1 {
                if let Ok(num) = val_str.parse::<i32>() {
                    enum1_values.push(NumberEnumValue {
                        value: num,
                        desc: desc.clone(),
                        since: None,
                    });
                }
            }
        }

        if !enum1_values.is_empty() {
            one_of_values.push(OneOfValue {
                typ: Value::NumberEnum(enum1_values),
                desc: None,
                since: type1_since,
            });
        }

        // Parse second type as enum (string)
        let mut enum2_values: Vec<StringEnumValue> = Vec::new();
        for (_, val2, desc) in &entries {
            if let Some(val_str) = val2 {
                // Clean up quoted strings
                let cleaned = if val_str.starts_with('"') && val_str.ends_with('"') {
                    val_str[1..val_str.len() - 1].to_string()
                } else {
                    val_str.clone()
                };
                enum2_values.push(StringEnumValue {
                    value: cleaned,
                    desc: desc.clone(),
                    since: None,
                });
            }
        }

        if !enum2_values.is_empty() {
            one_of_values.push(OneOfValue {
                typ: Value::StringEnum(enum2_values),
                desc: None,
                since: type2_since,
            });
        }
    }

    if one_of_values.is_empty() {
        return Ok(None);
    }

    Ok(Some(ParamItem {
        name: name_part.trim().to_string(),
        typ: Value::OneOf(one_of_values),
        desc: Some(desc),
        default: None,
        optional: false,
        since: None,
    }))
}

/// Try parsing a OneOf type parameter with explicit type descriptions
/// Format: name: [[Type1]] or [[Type2]] or ... :
/// * [[Type1]] - description1
/// * [[Type2]] - description2
pub fn try_oneof_types(source: &str) -> Result<Option<ParamItem>, String> {
    if !source.contains('\n') {
        return Ok(None);
    }

    let Some((name_part, type_and_description)) = source.split_once(": ") else {
        return Ok(None);
    };

    let lines = type_and_description.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return Ok(None);
    }

    let first_line = lines[0].trim();

    // Check if it matches the pattern: [[Type1]] or [[Type2]] ... :
    if !first_line.contains(" or ") || !first_line.ends_with(':') {
        return Ok(None);
    }

    // Collect the type-description pairs from bullet lines
    let mut type_descs: Vec<(String, Option<String>)> = Vec::new();

    for i in 1..lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('*') && !line.starts_with("**") {
            let line_content = line.trim_start_matches('*').trim();

            // Format: [[Type]] - description or just [[Type]]
            let (type_part, desc) = if let Some(pos) = line_content.find(" - ") {
                (
                    &line_content[..pos],
                    Some(line_content[pos + 3..].trim().to_string()),
                )
            } else {
                (line_content, None)
            };

            type_descs.push((type_part.trim().to_string(), desc));
        }
    }

    if type_descs.is_empty() {
        return Ok(None);
    }

    // Build OneOf values by parsing each type with its description
    let mut one_of_values: Vec<OneOfValue> = Vec::new();

    for (type_str, desc) in &type_descs {
        let typ = Value::parse(type_str, 0)?;

        // For complex types (containing " of "), include the type in the description
        let final_desc = if type_str.contains(" of ") {
            if let Some(d) = desc {
                Some(format!("{} - {}", type_str, d))
            } else {
                Some(type_str.clone())
            }
        } else {
            desc.clone()
        };

        one_of_values.push(OneOfValue {
            typ,
            desc: final_desc,
            since: None,
        });
    }

    // Remove unused type_descs warning if we're not using it directly in the final loop
    // (It's used indirectly through the iteration above)

    if one_of_values.is_empty() {
        return Ok(None);
    }

    // Extract overall description from the bullet points
    let overall_desc = if !one_of_values.is_empty() {
        let mut desc_parts = Vec::new();
        for oneof_val in &one_of_values {
            if let Some(desc) = &oneof_val.desc {
                desc_parts.push(desc.clone());
            }
        }
        if desc_parts.is_empty() {
            None
        } else {
            Some(desc_parts.join("\n"))
        }
    } else {
        None
    };

    Ok(Some(ParamItem {
        name: name_part.trim().to_string(),
        typ: Value::OneOf(one_of_values),
        desc: overall_desc,
        default: None,
        optional: false,
        since: None,
    }))
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
