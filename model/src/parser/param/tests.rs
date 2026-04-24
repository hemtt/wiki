use crate::{
    Arg, ArraySizedElement, Call, NumberEnumValue, OneOfValue, ParamItem, Since, StringEnumValue,
    Value, parser::param::try_optional,
};

#[test]
fn test_try_simple_line() {
    let line = "speed: [[Number]] - The speed of the vehicle.";
    let (param_item, errors) = ParamItem::parse("test", line).expect("Failed to parse simple line");
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
fn or() {
    let line = "return: [[String]] or [[Number]] - A string or number.";
    let (param_item, errors) =
        ParamItem::parse("test", line).expect("Failed to parse simple line with or");
    assert!(errors.is_empty());
    assert_eq!(param_item.name, "return");
    assert_eq!(param_item.desc.as_deref(), Some("A string or number."));
    assert_eq!(
        param_item.typ,
        Value::OneOf(vec![
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
        ])
    );
}

#[test]
fn array_of_or() {
    let line = "return: [[Array]] of [[String]]s or [[Number]]s - An array of strings or numbers.";
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
                        "index of mod in [[getLoadedModsInfo]] array. -1 if not found.".to_string()
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
fn array_with_columns() {
    let line = "return: [[Array]] with [rainDropTexture, texDropCount, minRainDensity, effectRadius, windCoef, dropSpeed, rndSpeed, rndDir, dropWidth, dropHeight, dropColor, lumSunFront, lumSunBack, refractCoef, refractSaturation, snow, dropColorStrong]
{{Columns|4|
* rainDropTexture: [[String]]
* texDropCount: [[Number]]
* minRainDensity: [[Number]]
* effectRadius: [[Number]]
* windCoef: [[Number]]
* dropSpeed: [[Number]]
* rndSpeed: [[Number]]
* rndDir: [[Number]]
* dropWidth: [[Number]]
* dropHeight: [[Number]]
* dropColor: [[Color|Color (RGBA)]]
* lumSunFront: [[Number]]
* lumSunBack: [[Number]]
* refractCoef: [[Number]]
* refractSaturation: [[Number]]
* snow: [[Boolean]]
* dropColorStrong: [[Boolean]]
}}";
    let (param_item, errors) =
        ParamItem::parse("test", line).expect("Failed to parse array with columns line");
    assert!(errors.is_empty());
    let Value::ArraySized(items) = param_item.typ else {
        panic!("Expected ArraySized");
    };
    assert_eq!(items.len(), 17);
    assert_eq!(items[0].name, "rainDropTexture");
    assert_eq!(items[13].name, "refractCoef");
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
fn array_with_nested_array_with() {
    let line = "return: [[Array]] of [[Array]]s with [magazineName, muzzleName, id, ammoCount]
* magazineName: [[String]]
* muzzleName: [[String]]
* id: [[Array]] with [itemWorldID, creatorID]
** itemWorldID: [[Number]] - unique ID of the item in the world
** creatorID: [[Number]] - unique ID of the player who created the item
* ammoCount: [[Number]] magazine ammo";
    let (param_item, errors) =
        ParamItem::parse("test", line).expect("Failed to parse nested array with array with line");
    assert!(errors.is_empty());
    assert_eq!(
        param_item.typ,
        Value::ArrayUnsized {
            value: Box::new(Value::ArraySized(vec![
                ArraySizedElement {
                    name: "magazineName".to_string(),
                    typ: Value::String,
                    default: None,
                    desc: None,
                    since: None,
                },
                ArraySizedElement {
                    name: "muzzleName".to_string(),
                    typ: Value::String,
                    default: None,
                    desc: None,
                    since: None,
                },
                ArraySizedElement {
                    name: "id".to_string(),
                    typ: Value::ArraySized(vec![
                        ArraySizedElement {
                            name: "itemWorldID".to_string(),
                            typ: Value::Number,
                            default: None,
                            desc: Some("unique ID of the item in the world".to_string()),
                            since: None,
                        },
                        ArraySizedElement {
                            name: "creatorID".to_string(),
                            typ: Value::Number,
                            default: None,
                            desc: Some("unique ID of the player who created the item".to_string()),
                            since: None,
                        },
                    ]),
                    default: None,
                    desc: None,
                    since: None,
                },
                ArraySizedElement {
                    name: "ammoCount".to_string(),
                    typ: Value::Number,
                    default: None,
                    desc: Some("magazine ammo".to_string()),
                    since: None,
                },
            ]))
        }
    );
}

#[test]
fn number_enum() {
    let line = "return: [[Number]] - admin state of given client:
* 0 - client is not an admin
* 1 - client is admin that is [[Arma 3: Server Config File#Voted_In_Admin|voted in]]
* 2 - client is admin that is [[Arma 3: Server Config File#Logged_In_Admin|logged in]]";
    let (param_item, errors) =
        ParamItem::parse("test", line).expect("Failed to parse number enum return");
    assert!(errors.is_empty());
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

    let line = "index: [[Number]] - from 1 to 10

* 1 - Alpha (key 1)
* 2 - Bravo (key 2)
* 3 - Charlie (key 3)
* 4 - Delta (key 4)
* 5 - Echo (key 5)
* 6 - Foxtrot (key 6)
* 7 - Golf (key 7)
* 8 - Hotel (key 8)
* 9 - India (key 9)
* 10 - Juliet (key 0)";
    let (param_item, errors) =
        ParamItem::parse("test", line).expect("Failed to parse number enum index");
    assert!(errors.is_empty());
    assert_eq!(param_item.name, "index");
    assert_eq!(param_item.desc.as_deref(), Some("from 1 to 10"));
    assert_eq!(
        param_item.typ,
        Value::NumberEnum(vec![
            NumberEnumValue {
                value: 1,
                desc: Some("Alpha (key 1)".to_string()),
                since: None,
            },
            NumberEnumValue {
                value: 2,
                desc: Some("Bravo (key 2)".to_string()),
                since: None,
            },
            NumberEnumValue {
                value: 3,
                desc: Some("Charlie (key 3)".to_string()),
                since: None,
            },
            NumberEnumValue {
                value: 4,
                desc: Some("Delta (key 4)".to_string()),
                since: None,
            },
            NumberEnumValue {
                value: 5,
                desc: Some("Echo (key 5)".to_string()),
                since: None,
            },
            NumberEnumValue {
                value: 6,
                desc: Some("Foxtrot (key 6)".to_string()),
                since: None,
            },
            NumberEnumValue {
                value: 7,
                desc: Some("Golf (key 7)".to_string()),
                since: None,
            },
            NumberEnumValue {
                value: 8,
                desc: Some("Hotel (key 8)".to_string()),
                since: None,
            },
            NumberEnumValue {
                value: 9,
                desc: Some("India (key 9)".to_string()),
                since: None,
            },
            NumberEnumValue {
                value: 10,
                desc: Some("Juliet (key 0)".to_string()),
                since: None,
            },
        ])
    );
}

#[test]
fn string_enum() {
    let line = r#"shape: [[String]] - the shape, can be one of:
* {{hl|"ICON"}}
* {{hl|"RECTANGLE"}}
* {{hl|"ELLIPSE"}}
* {{GVI|arma3|1.60|size= 0.75}} {{hl|"POLYLINE"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"TRIANGLE"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"PENTAGON"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"HEXAGON"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"HEPTAGON"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"OCTAGON"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"NONAGON"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"DECAGON"}}"#;
    let (param_item, errors) =
        ParamItem::parse("test", line).expect("Failed to parse string enum line");
    assert!(errors.is_empty());
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
                since: Some(Since::arma3("1.60")),
            },
            StringEnumValue {
                value: "TRIANGLE".to_string(),
                desc: None,
                since: Some(Since::arma3("2.20")),
            },
            StringEnumValue {
                value: "PENTAGON".to_string(),
                desc: None,
                since: Some(Since::arma3("2.20")),
            },
            StringEnumValue {
                value: "HEXAGON".to_string(),
                desc: None,
                since: Some(Since::arma3("2.20")),
            },
            StringEnumValue {
                value: "HEPTAGON".to_string(),
                desc: None,
                since: Some(Since::arma3("2.20")),
            },
            StringEnumValue {
                value: "OCTAGON".to_string(),
                desc: None,
                since: Some(Since::arma3("2.20")),
            },
            StringEnumValue {
                value: "NONAGON".to_string(),
                desc: None,
                since: Some(Since::arma3("2.20")),
            },
            StringEnumValue {
                value: "DECAGON".to_string(),
                desc: None,
                since: Some(Since::arma3("2.20")),
            },
        ])
    );

    let line = r#"return: [[String]] - unit behaviour - one of:
{{Columns|2|
* "CARELESS"
* "SAFE"
* "AWARE"
* "COMBAT"
* "STEALTH"
* "ERROR" - when not available
}}"#;
    let (param_item, errors) =
        ParamItem::parse("test", line).expect("Failed to parse string enum with columns line");
    assert!(errors.is_empty());
    assert_eq!(param_item.name, "return");
    assert_eq!(param_item.desc, Some("unit behaviour".to_string()));
    assert_eq!(
        param_item.typ,
        Value::StringEnum(vec![
            StringEnumValue {
                value: "CARELESS".to_string(),
                desc: None,
                since: None,
            },
            StringEnumValue {
                value: "SAFE".to_string(),
                desc: None,
                since: None,
            },
            StringEnumValue {
                value: "AWARE".to_string(),
                desc: None,
                since: None,
            },
            StringEnumValue {
                value: "COMBAT".to_string(),
                desc: None,
                since: None,
            },
            StringEnumValue {
                value: "STEALTH".to_string(),
                desc: None,
                since: None,
            },
            StringEnumValue {
                value: "ERROR".to_string(),
                desc: Some("when not available".to_string()),
                since: None,
            },
        ])
    );

    let line = r#"toneName: [[String]] - can be one of:
* "Arma"
* "Filmic"
* "Reinhard""#;
    let (param_item, errors) = ParamItem::parse("test", line)
        .expect("Failed to parse string enum with incomplete list line");
    assert!(errors.is_empty());
    assert_eq!(param_item.name, "toneName");
    assert_eq!(param_item.desc, Some(String::new()));
    assert_eq!(
        param_item.typ,
        Value::StringEnum(vec![
            StringEnumValue {
                value: "Arma".to_string(),
                desc: None,
                since: None,
            },
            StringEnumValue {
                value: "Filmic".to_string(),
                desc: None,
                since: None,
            },
            StringEnumValue {
                value: "Reinhard".to_string(),
                desc: None,
                since: None,
            },
        ])
    );
}

#[test]
fn multiple_type_enum() {
    let line = r#"mode: [[Number]] or [[String]] - can be one of the following:
* 0 / "AUTO" - control position (x, y, w, h) is always rounded to whole pixels (game default)
* 1 / "ON" - control position (x, y, w, h) is rounded to whole pixels only when not animating
* 2 / "OFF" - control position (x, y, w, h) is always precise, no pixel rounding applies here"#;
    let (param_item, errors) =
        ParamItem::parse("test", line).expect("Failed to parse multiple type enum line");
    assert!(errors.is_empty());
    assert_eq!(param_item.name, "mode");
    assert_eq!(
        param_item.desc.as_deref(),
        Some("can be one of the following")
    );
    assert_eq!(
        param_item.typ,
        Value::OneOf(vec![
            OneOfValue {
                typ: Value::NumberEnum(vec![
                    NumberEnumValue {
                        value: 0,
                        desc: Some("control position (x, y, w, h) is always rounded to whole pixels (game default)".to_string()),
                        since: None,
                    },
                    NumberEnumValue {
                        value: 1,
                        desc: Some("control position (x, y, w, h) is rounded to whole pixels only when not animating".to_string()),
                        since: None,
                    },
                    NumberEnumValue {
                        value: 2,
                        desc: Some("control position (x, y, w, h) is always precise, no pixel rounding applies here".to_string()),
                        since: None,
                    },
                ]),
                desc: None,
                since: None,
            },
            OneOfValue {
                typ: Value::StringEnum(vec![
                    StringEnumValue {
                        value: "AUTO".to_string(),
                        desc: Some("control position (x, y, w, h) is always rounded to whole pixels (game default)".to_string()),
                        since: None,
                    },
                    StringEnumValue {
                        value: "ON".to_string(),
                        desc: Some("control position (x, y, w, h) is rounded to whole pixels only when not animating".to_string()),
                        since: None,
                    },
                    StringEnumValue {
                        value: "OFF".to_string(),
                        desc: Some("control position (x, y, w, h) is always precise, no pixel rounding applies here".to_string()),
                        since: None,
                    },
                ]),
                desc: None,
                since: None,
            },
        ])
    );

    let line = r#"isSpeech: [[Boolean]] or {{GVI|arma3|2.04|size= 0.75}} [[Number]] - (Optional, default [[false]])
* 0/[[false]] = play as sound ([[fadeSound]] applies)
* 1/[[true]] = play as speech ([[fadeSpeech]] applies), filters are not applied to it (i.e. house or vehicle interior one)
* 2 = play as sound ([[fadeSound]] applies) without interior/vehicle muffling"#;
    let (param_item, errors) =
        ParamItem::parse("test", line).expect("Failed to parse multiple type enum line");
    assert!(errors.is_empty());
    assert_eq!(param_item.name, "isSpeech");
    assert_eq!(
        param_item.desc.as_deref(),
        Some("(Optional, default [[false]])")
    );
    assert_eq!(
        param_item.typ,
        Value::OneOf(vec![
            OneOfValue {
                typ: Value::Boolean,
                desc: Some("0/[[false]] = play as sound ([[fadeSound]] applies)\n1/[[true]] = play as speech ([[fadeSpeech]] applies), filters are not applied to it (i.e. house or vehicle interior one)\n2 = play as sound ([[fadeSound]] applies) without interior/vehicle muffling".to_string()),
                since: None,
            },
            OneOfValue {
                typ: Value::NumberEnum(vec![
                    NumberEnumValue {
                        value: 0,
                        desc: Some("play as sound ([[fadeSound]] applies)".to_string()),
                        since: None,
                    },
                    NumberEnumValue {
                        value: 1,
                        desc: Some("play as speech ([[fadeSpeech]] applies), filters are not applied to it (i.e. house or vehicle interior one)".to_string()),
                        since: None,
                    },
                    NumberEnumValue {
                        value: 2,
                        desc: Some("play as sound ([[fadeSound]] applies) without interior/vehicle muffling".to_string()),
                        since: None,
                    },
                ]),
                desc: None,
                since: Some(Since::arma3("2.04")),
            },
        ])
    );
}

#[test]
fn extra_description() {
    let line = r#"killer: [[Object]] - (Optional, default [[objNull]]) the entity that caused the damage. If the damage leads to the death of the unit, the killer will be used as the object that caused the kill.
* it can be used to show "killed by player" in debriefing statistics and kill messages in the chat (if death messages are enabled).
* it will alter the killer's [[rating]] as if the killer directly killed the unit.
* it will be listed as <sqf inline>_killer</sqf> parameter in the [[Arma 3: Event Handlers#Killed|Killed]] event handler."#;
    let (param_item, errors) =
        ParamItem::parse("test", line).expect("Failed to parse line with extra description");
    assert!(errors.is_empty());
    assert_eq!(param_item.name, "killer");
    assert_eq!(
        param_item.desc,
        Some("the entity that caused the damage. If the damage leads to the death of the unit, the killer will be used as the object that caused the kill.\n* it can be used to show \"killed by player\" in debriefing statistics and kill messages in the chat (if death messages are enabled).\n* it will alter the killer's [[rating]] as if the killer directly killed the unit.\n* it will be listed as <sqf inline>_killer</sqf> parameter in the [[Arma 3: Event Handlers#Killed|Killed]] event handler.".to_string())
    );

    let line = r#"texture: [[String]] - flag texture. If texture is {{hl|""}}, flag is not drawn. Custom texture can be used:
* Prior {{arma3}}: Dimension 200x200, file extension .jpg
* Since {{arma3}}: Dimension 512×256, file extension .paa"#;
    let (param_item, errors) =
        ParamItem::parse("test", line).expect("Failed to parse line with extra description");
    assert!(errors.is_empty());
    assert_eq!(param_item.name, "texture");
    assert_eq!(
        param_item.desc,
        Some("flag texture. If texture is {{hl|\"\"}}, flag is not drawn. Custom texture can be used:\n* Prior {{arma3}}: Dimension 200x200, file extension .jpg\n* Since {{arma3}}: Dimension 512×256, file extension .paa".to_string())
    );
}

#[test]
fn or_array_of() {
    let line = "thenCode: [[Code]] or [[Array]] of [[Code]]:
* [[Code]] - code block to execute if ''ifType''<nowiki/>'s condition is [[true]]
* [[Array]] of [[Code]] - array of two [[Code]] elements in format [thenCode, elseCode]; see {{Link|#Example 3}}";
    let (param_item, errors) =
        ParamItem::parse("test", line).expect("Failed to parse or array of line");
    assert!(errors.is_empty());
    assert_eq!(param_item.name, "thenCode");
    assert_eq!(
        param_item.desc.as_deref(),
        Some(
            "code block to execute if ''ifType''<nowiki/>'s condition is [[true]]\n[[Array]] of [[Code]] - array of two [[Code]] elements in format [thenCode, elseCode]; see {{Link|#Example 3}}"
        )
    );
    assert_eq!(
        param_item.typ,
        Value::OneOf(vec![
            OneOfValue {
                typ: Value::Code,
                desc: Some("code block to execute if ''ifType''<nowiki/>'s condition is [[true]]".to_string()),
                since: None,
            },
            OneOfValue {
                typ: Value::ArrayUnsized {
                    value: Box::new(Value::Code),
                },
                desc: Some("[[Array]] of [[Code]] - array of two [[Code]] elements in format [thenCode, elseCode]; see {{Link|#Example 3}}".to_string()),
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
