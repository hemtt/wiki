use arma3_wiki_model::{Call, Param, ParamItem, Since, Value};

use crate::parser::{ParseError, call::CallParser, value::ValueParser};

pub trait ParamItemParser {
    fn parse(command: &str, source: &str) -> Result<(Self, Vec<ParseError>), String>
    where
        Self: Sized;
}

impl ParamItemParser for ParamItem {
    fn parse(command: &str, source: &str) -> Result<(Self, Vec<ParseError>), String> {
        if let Some(parsed) = try_simple_line(source)? {
            return Ok((parsed, Vec::new()));
        }
        if let Some(parsed) = try_array_with(source)? {
            return Ok((parsed, Vec::new()));
        }
        Err(format!(
            "Failed to parse parameter for command '{command}': '{source}'"
        ))
    }
}

/// Try parsing a simple line parameter
/// name: [[Type]] - Description
pub fn try_simple_line(source: &str) -> Result<Option<ParamItem>, String> {
    if source.contains('\n') {
        return Ok(None);
    }
    let (since, source) = if source.starts_with("{{") {
        super::extract_since(source)?
    } else {
        (None, source)
    };
    let Some((name_part, type_and_description)) = source.split_once(": ") else {
        return Ok(None);
    };
    let (type_part, desc) =
        if let Some((type_part, description_part)) = type_and_description.split_once(" - ") {
            (type_part, Some(description_part.trim().to_string()))
        } else {
            (type_and_description, None)
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
    for line in lines {
        let line = line.trim().trim_start_matches('*').trim().to_string();
        // detect index, eg: 0 - {name}: [[Type]] - Description
        let line = if let Some((index, rest)) = line.split_once(" - ") {
            if index.trim().chars().all(|c| c.is_ascii_digit()) {
                rest.trim().to_string()
            } else {
                line
            }
        } else {
            line
        };
        if let Ok(Some(item)) = try_simple_line(&line) {
            params.push(item);
        } else {
            return Err(format!("Failed to parse array with element line: '{line}'"));
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

#[must_use]
/// Try to determine if the parameter is optional from description
///
/// # Examples
/// The item's class name.                      -> None
/// (Optional, default 5) The number of items.  -> Some((Some(Number(5)), "The number of items."))
/// (Optional) The name of the item.            -> Some((None, "The name of the item."))
pub fn try_optional(source: &str) -> Option<(Option<String>, String)> {
    let source_lower = source.trim().to_lowercase();
    if source_lower.starts_with("(optional") {
        if let Some(default_start) = source_lower.find("default ") {
            let default_end = source_lower[default_start..]
                .find(')')
                .map_or(source_lower.len(), |i| default_start + i);
            let default_str = source_lower[default_start + 8..default_end]
                .trim()
                .trim_start_matches("[[")
                .trim_end_matches("]]");
            return Some((
                Some(default_str.to_string()),
                source[default_end + 1..].trim().to_string(),
            ));
        }
        return Some((None, source.trim().to_string()));
    }
    None
}

#[cfg(test)]
mod tests {
    use arma3_wiki_model::{Arg, ArraySizedElement, OneOfValue, Since};

    use super::*;

    #[test]
    fn test_try_simple_line() {
        let line = "speed: [[Number]] - The speed of the vehicle.";
        let (param_item, errors) =
            ParamItem::parse("test", line).expect("Failed to parse simple line");
        assert!(errors.is_empty());
        assert_eq!(param_item.name, "speed");
        assert_eq!(
            param_item.desc.as_deref(),
            Some("The speed of the vehicle.")
        );

        let line = "x: [[Number]] in range -1..+1 - any other value returns [[NaN]]";
        let (param_item, errors) =
            ParamItem::parse("test", line).expect("Failed to parse simple line with range");
        assert!(errors.is_empty());
        assert_eq!(param_item.name, "x");
        assert_eq!(
            param_item.desc.as_deref(),
            Some("any other value returns [[NaN]]")
        );
        assert_eq!(param_item.typ, Value::NumberRange(-1, 1));
    }

    #[test]
    fn array_of_or() {
        let line =
            "return: [[Array]] of [[String]]s or [[Number]]s - An array of strings or numbers.";
        let (param_item, errors) =
            ParamItem::parse("test", line).expect("Failed to parse array of line");
        assert!(errors.is_empty());
        assert_eq!(param_item.name, "return");
        assert_eq!(
            param_item.desc.as_deref(),
            Some("An array of strings or numbers.")
        );
        assert_eq!(
            param_item.typ,
            Value::ArrayUnsized {
                value: Box::new(Value::OneOf(vec![
                    OneOfValue {
                        typ: Value::String,
                        desc: None,
                        since: None,
                    },
                    OneOfValue {
                        typ: Value::Number,
                        desc: None,
                        since: None,
                    }
                ]))
            }
        );
    }

    #[test]
    fn array_of_or_many() {
        let line = "return: [[Array]] of [[String]]s, [[Number]]s, [[Object]]s, [[Waypoint]]s, or [[Group]]s - An array of strings, numbers, objects, waypoints, or groups.";
        let (param_item, errors) =
            ParamItem::parse("test", line).expect("Failed to parse array of line");
        assert!(errors.is_empty());
        assert_eq!(param_item.name, "return");
        assert_eq!(
            param_item.desc.as_deref(),
            Some("An array of strings, numbers, objects, waypoints, or groups.")
        );
        assert_eq!(
            param_item.typ,
            Value::ArrayUnsized {
                value: Box::new(Value::OneOf(vec![
                    OneOfValue {
                        typ: Value::String,
                        desc: None,
                        since: None,
                    },
                    OneOfValue {
                        typ: Value::Number,
                        desc: None,
                        since: None,
                    },
                    OneOfValue {
                        typ: Value::Object,
                        desc: None,
                        since: None,
                    },
                    OneOfValue {
                        typ: Value::Waypoint,
                        desc: None,
                        since: None,
                    },
                    OneOfValue {
                        typ: Value::Group,
                        desc: None,
                        since: None,
                    }
                ]))
            }
        );
    }

    #[test]
    fn test_try_optional() {
        let line_with_default = "(Optional, default 10) The number of items.";
        let optional_value =
            try_optional(line_with_default).expect("Failed to parse optional with default");
        assert_eq!(
            optional_value,
            (Some("10".to_string()), "The number of items.".to_string())
        );
        // Further assertions on optional_value can be added here

        let line_without_default = "(Optional) The name of the item.";
        let optional_value =
            try_optional(line_without_default).expect("Failed to parse optional without default");
        assert_eq!(optional_value, (None, "The name of the item.".to_string()));

        let non_optional_line = "The item's class name.";
        let optional_value = try_optional(non_optional_line);
        assert_eq!(optional_value, None);
    }

    #[test]
    fn array_with() {
        let line = "return: [[Array]] with [condition, statement] - Details about the waypoint
* condition: [[String]]
* statement: [[String]]";
        let (param_item, errors) =
            ParamItem::parse("test", line).expect("Failed to parse array with line");
        assert!(errors.is_empty());
        assert_eq!(param_item.name, "return");
        assert_eq!(
            param_item.desc.as_deref(),
            Some("Details about the waypoint")
        );
        assert_eq!(
            param_item.typ,
            Value::ArraySized(vec![
                ArraySizedElement {
                    name: "condition".to_string(),
                    typ: Value::String,
                    default: None,
                    desc: None,
                    since: None,
                },
                ArraySizedElement {
                    name: "statement".to_string(),
                    typ: Value::String,
                    default: None,
                    desc: None,
                    since: None,
                },
            ],)
        );

        let line = "return: [[Array]] with [ambientLife, ambientSound, windyCoef]
* ambientLife: [[Boolean]] 
* ambientSound: [[Boolean]]
* {{GVI|arma3|2.12|size= 0.75}} windyCoef: [[Number]] - see [[enableEnvironment]]";
        let (param_item, errors) =
            ParamItem::parse("test", line).expect("Failed to parse array with line");
        assert!(errors.is_empty());
        assert_eq!(
            param_item.typ,
            Value::ArraySized(vec![
                ArraySizedElement {
                    name: "ambientLife".to_string(),
                    typ: Value::Boolean,
                    default: None,
                    desc: None,
                    since: None,
                },
                ArraySizedElement {
                    name: "ambientSound".to_string(),
                    typ: Value::Boolean,
                    default: None,
                    desc: None,
                    since: None,
                },
                ArraySizedElement {
                    name: "windyCoef".to_string(),
                    typ: Value::Number,
                    default: None,
                    desc: Some("see [[enableEnvironment]]".to_string()),
                    since: Some(Since::arma3("2.12")),
                },
            ])
        );

        let line = "return: [[Array]] with [isMan, isAnimal]
* 0 - isMan: [[Boolean]] - [[true]] if the entity is a man
* 1 - isAnimal: [[Boolean]] - [[true]] if the entity is an animal";
        let (param_item, errors) =
            ParamItem::parse("test", line).expect("Failed to parse array with indexed line");
        assert!(errors.is_empty());
        assert_eq!(
            param_item.typ,
            Value::ArraySized(vec![
                ArraySizedElement {
                    name: "isMan".to_string(),
                    typ: Value::Boolean,
                    default: None,
                    desc: Some("[[true]] if the entity is a man".to_string()),
                    since: None,
                },
                ArraySizedElement {
                    name: "isAnimal".to_string(),
                    typ: Value::Boolean,
                    default: None,
                    desc: Some("[[true]] if the entity is an animal".to_string()),
                    since: None,
                },
            ])
        );

        let line = "retrun: [[Array]] of [[Array]]s with [prefix, version, isPatched, modIndex, hash]
* prefix: [[String]] - addon prefix
* version: [[String]] - addon revision version
* isPatched: [[Boolean]] - [[true]] if patching is enabled and this addon is being patched 
* {{GVI|arma3|2.14|size= 0.75}} modIndex: [[Number]] - index of mod in [[getLoadedModsInfo]] array. -1 if not found.
* {{GVI|arma3|2.14|size= 0.75}} hash: [[String]] - hash of the addon PBO file.";
        let (param_item, errors) =
            ParamItem::parse("test", line).expect("Failed to parse nested array with line");
        assert!(errors.is_empty());
        assert_eq!(
            param_item.typ,
            Value::ArrayUnsized {
                value: Box::new(Value::ArraySized(vec![
                    ArraySizedElement {
                        name: "prefix".to_string(),
                        typ: Value::String,
                        default: None,
                        desc: Some("addon prefix".to_string()),
                        since: None,
                    },
                    ArraySizedElement {
                        name: "version".to_string(),
                        typ: Value::String,
                        default: None,
                        desc: Some("addon revision version".to_string()),
                        since: None,
                    },
                    ArraySizedElement {
                        name: "isPatched".to_string(),
                        typ: Value::Boolean,
                        default: None,
                        desc: Some(
                            "[[true]] if patching is enabled and this addon is being patched"
                                .to_string()
                        ),
                        since: None,
                    },
                    ArraySizedElement {
                        name: "modIndex".to_string(),
                        typ: Value::Number,
                        default: None,
                        desc: Some(
                            "index of mod in [[getLoadedModsInfo]] array. -1 if not found."
                                .to_string()
                        ),
                        since: Some(Since::arma3("2.14")),
                    },
                    ArraySizedElement {
                        name: "hash".to_string(),
                        typ: Value::String,
                        default: None,
                        desc: Some("hash of the addon PBO file.".to_string()),
                        since: Some(Since::arma3("2.14")),
                    },
                ]))
            }
        );
    }

    #[test]
    fn array_with_default() {
        let line = "args: [[Array]] with [setIsSanta, setIsGhost]
* setIsSanta: [[Boolean]] - (Optional, default [[false]]) Set to [[true]] to make the entity a Santa.
* setIsGhost: [[Boolean]] - (Optional, default [[false]]) Set to [[true]] to make the entity a ghost.";
        let (param_item, errors) =
            ParamItem::parse("test", line).expect("Failed to parse array with indexed line");
        assert!(errors.is_empty());
        assert_eq!(
            param_item.typ,
            Value::ArraySized(vec![
                ArraySizedElement {
                    name: "setIsSanta".to_string(),
                    typ: Value::Boolean,
                    default: Some("false".to_string()),
                    desc: Some("Set to [[true]] to make the entity a Santa.".to_string()),
                    since: None,
                },
                ArraySizedElement {
                    name: "setIsGhost".to_string(),
                    typ: Value::Boolean,
                    default: Some("false".to_string()),
                    desc: Some("Set to [[true]] to make the entity a ghost.".to_string()),
                    since: None,
                },
            ])
        );
    }

    #[test]
    fn parse() {
        assert_eq!(
            Call::parse_params("[idc, path, name]").expect("Invalid parameters"),
            Arg::Array(vec![
                Arg::Item("idc".to_string()),
                Arg::Item("path".to_string()),
                Arg::Item("name".to_string())
            ])
        );
        assert_eq!(
            Call::parse_params("[idc, [row, column], colour]").expect("Invalid parameters"),
            Arg::Array(vec![
                Arg::Item("idc".to_string()),
                Arg::Array(vec![
                    Arg::Item("row".to_string()),
                    Arg::Item("column".to_string())
                ]),
                Arg::Item("colour".to_string())
            ])
        );
        assert_eq!(
            Call::parse_params("[[row, column], colour]").expect("Invalid parameters"),
            Arg::Array(vec![
                Arg::Array(vec![
                    Arg::Item("row".to_string()),
                    Arg::Item("column".to_string())
                ]),
                Arg::Item("colour".to_string())
            ])
        );
    }
}
