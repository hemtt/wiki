use crate::Since;

/// Strip wiki markup from text
/// Replaces [[link|text]] with text and [[text]] with text
pub fn strip_wiki_markup(text: &str) -> String {
    let mut result = text.to_string();

    // Replace [[link|text]] with text and [[text]] with text
    while let Some(start) = result.find("[[") {
        if let Some(end) = result[start + 2..].find("]]") {
            let end = start + 2 + end;
            let inside = &result[start + 2..end];

            if let Some(pipe_pos) = inside.find('|') {
                // [[link|text]] -> text
                let replacement = inside[pipe_pos + 1..].to_string();
                result = format!("{}{}{}", &result[..start], replacement, &result[end + 2..]);
            } else {
                // [[text]] -> text
                result = format!("{}{}{}", &result[..start], inside, &result[end + 2..]);
            }
        } else {
            break;
        }
    }

    result
}

/// Generic enum parser helper for extracting enum values from bullet-pointed lines
/// Handles optional version info and applies a custom value extractor function
pub fn parse_enum_lines<'a, F>(
    lines: std::str::Lines<'a>,
    value_extractor: F,
) -> Vec<(String, Option<Since>)>
where
    F: Fn(&str) -> Option<String>,
{
    let mut enum_values: Vec<(String, Option<Since>)> = Vec::new();

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
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

            // Try to extract the value using the provided extractor function
            if let Some(value) = value_extractor(content_for_value) {
                enum_values.push((value, since));
            }
        }
    }

    enum_values
}

/// Extract a quoted string value from {{hl|"VALUE"}} format or plain "VALUE" format
/// Also extracts optional description after " - "
pub fn extract_string_enum_value(text: &str) -> Option<String> {
    // Try {{hl|\"VALUE\"}} format first
    if let Some(start) = text.find("{{hl|\"") {
        let after_prefix = &text[start + 6..];
        if let Some(end) = after_prefix.find("\"") {
            return Some(after_prefix[..end].to_string());
        }
    }

    // Try plain \"VALUE\" format (just quoted string at start of line)
    let trimmed = text.trim_start();
    if trimmed.starts_with('\"') {
        if let Some(end) = trimmed[1..].find('\"') {
            return Some(trimmed[1..end + 1].to_string());
        }
    }

    None
}

/// Extract string enum value with optional description
/// Returns (value, description)
pub fn extract_string_enum_value_with_desc(text: &str) -> Option<(String, Option<String>)> {
    let mut value = None;
    let mut remainder = text.trim_start();

    // Try {{hl|\"VALUE\"}} format first
    if let Some(start) = remainder.find("{{hl|\"") {
        let after_prefix = &remainder[start + 6..];
        if let Some(end) = after_prefix.find("\"") {
            value = Some(after_prefix[..end].to_string());
            remainder = &remainder[start + 8 + end..];
        }
    }

    // If not found, try plain \"VALUE\" format
    if value.is_none() && remainder.starts_with('\"') {
        if let Some(end) = remainder[1..].find('\"') {
            value = Some(remainder[1..end + 1].to_string());
            remainder = &remainder[end + 2..];
        }
    }

    // Extract description if present
    let desc = if remainder.trim_start().starts_with(" - ") {
        let desc_str = remainder.trim_start()[3..].trim();
        if !desc_str.is_empty() {
            Some(desc_str.to_string())
        } else {
            None
        }
    } else if remainder.trim_start().starts_with('-') {
        let desc_str = remainder.trim_start()[1..].trim();
        if !desc_str.is_empty() {
            Some(desc_str.to_string())
        } else {
            None
        }
    } else {
        None
    };

    value.map(|v| (v, desc))
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
        // If no default, find closing ) and skip past it
        if let Some(close_paren) = source_lower.find(')') {
            return Some((None, source[close_paren + 1..].trim().to_string()));
        }
        return Some((None, source.trim().to_string()));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_wiki_markup() {
        assert_eq!(strip_wiki_markup("[[link|text]]"), "text");
        assert_eq!(strip_wiki_markup("[[text]]"), "text");
        assert_eq!(
            strip_wiki_markup("before [[link|inner]] after"),
            "before inner after"
        );
        assert_eq!(
            strip_wiki_markup("multiple [[a|one]] and [[b|two]]"),
            "multiple one and two"
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
    fn test_extract_string_enum_value() {
        assert_eq!(
            extract_string_enum_value(r#"{{hl|"ICON"}}"#),
            Some("ICON".to_string())
        );
        assert_eq!(
            extract_string_enum_value(r#"{{hl|"VALUE"}}"#),
            Some("VALUE".to_string())
        );
        assert_eq!(extract_string_enum_value("no match"), None);
    }
}
