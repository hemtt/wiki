mod helpers;
mod parsers;

#[cfg(test)]
mod tests;

use crate::ParamItem;

// Re-export public helpers that might be useful
pub use helpers::{extract_string_enum_value, parse_enum_lines, strip_wiki_markup, try_optional};

// Re-export parsers for internal use
use parsers::{
    try_array_with, try_number_enum, try_simple_line, try_simple_line_with_extra_description,
    try_string_enum,
};

/// Main dispatcher for parsing parameter definitions
impl ParamItem {
    pub fn parse(
        command: &str,
        source: &str,
    ) -> Result<(Self, Vec<crate::parser::ParseError>), String> {
        if let Some(parsed) = try_simple_line(source)? {
            return Ok((parsed, Vec::new()));
        }
        if let Some(parsed) = try_array_with(source)? {
            return Ok((parsed, Vec::new()));
        }
        if let Some(parsed) = try_number_enum(source)? {
            return Ok((parsed, Vec::new()));
        }
        if let Some(parsed) = try_string_enum(source)? {
            return Ok((parsed, Vec::new()));
        }
        if let Some(parsed) = try_simple_line_with_extra_description(source)? {
            return Ok((parsed, Vec::new()));
        }
        Err(format!(
            "Failed to parse parameter for command '{command}': '{source}'"
        ))
    }
}
