use regex::Regex;

pub enum ParsedLine<'a> {
    QueryNo(&'a str),
    Bind(&'a str),
    Query(&'a str),
    End,
    Filename(&'a str),
}

pub fn parse_line<'a>(
    line: &'a str,
    re_query_no: &Regex,
    re_bind: &Regex,
    re_query: &Regex,
    re_end: &Regex,
    re_filename: &Regex,
) -> Option<ParsedLine<'a>> {
    if let Some(caps) = re_query_no.captures(line) {
        // Note: using .get(1) is a bit safer than indexing.
        if let Some(m) = caps.get(1) {
            return Some(ParsedLine::QueryNo(m.as_str()));
        }
    } else if let Some(mat) = re_bind.find(line) {
        // Everything after the match is the captured text.
        let captured_text = &line[mat.end()..];
        return Some(ParsedLine::Bind(captured_text));
    } else if let Some(caps) = re_query.captures(line) {
        if let Some(m) = caps.get(1) {
            return Some(ParsedLine::Query(m.as_str()));
        }
    } else if re_end.is_match(line) {
        return Some(ParsedLine::End);
    } else if let Some(caps) = re_filename.captures(line) {
        if let Some(m) = caps.get(1) {
            return Some(ParsedLine::Filename(m.as_str()));
        }
    }
    None
}

use anyhow::{bail, Result};

/// Parses a single line and returns its value as a [`String`].
///
/// The function handles the following cases:
/// - `"NULL"`: returns `"NULL"`
/// - `"SHORT <value>"`: returns the `<value>` (ignoring numeric parsing)
/// - `"INT <value>"`: returns the `<value>`
/// - `"VARCHAR (<number>)<value>"`: returns `<value>`, where `<number>` is
///
/// expected to be within parentheses and is ignored.
///
/// # Errors
///
/// Returns an error if the input format is unrecognized or if the VARCHAR format is malformed.
pub fn parse_bind_value(line: &str) -> Result<String> {
    let trimmed = line.trim();

    if trimmed == "NULL" {
        Ok("NULL".to_string())
    } else if trimmed.starts_with("SHORT ") {
        // Extract everything after "SHORT " and return as string.
        let value_str = trimmed
            .strip_prefix("SHORT ")
            .expect("Prefix should exist")
            .trim();
        Ok(value_str.to_string())
    } else if trimmed.starts_with("INT ") {
        // Extract everything after "INT " and return as string.
        let value_str = trimmed
            .strip_prefix("INT ")
            .expect("Prefix should exist")
            .trim();
        Ok(value_str.to_string())
    } else if trimmed.starts_with("VARCHAR") {
        // Expected format: "VARCHAR (<number>)<value>"
        // Find the first occurrence of ')' to separate the length info from the value.
        if let Some(close_paren_idx) = trimmed.find(')') {
            let content = trimmed[close_paren_idx + 1..].trim();
            Ok(content.to_string())
        } else {
            bail!("Malformed VARCHAR field: missing ')'");
        }
    } else {
        bail!("Unrecognized field format");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_parse_line() -> Result<()> {
        // Each tuple consists of (input line, expected output string).
        let test_cases = vec![
            ("NULL", "NULL"),
            ("SHORT 0", "0"),
            ("VARCHAR (11)J100002422", "J100002422"),
            (
                "VARCHAR (33)f8cd7be60d774153a67f720aaec243e0",
                "f8cd7be60d774153a67f720aaec243e0",
            ),
            ("VARCHAR (10)경기도", "경기도"),
            ("VARCHAR (10)중학교", "중학교"),
            ("INT 0", "0"),
            ("NULL", "NULL"),
            ("VARCHAR (11)383.3125px", "383.3125px"),
            ("VARCHAR (13)131.086875px", "131.086875px"),
            (
                "VARCHAR (30)115.14388275146484|292.28125|",
                "115.14388275146484|292.28125|",
            ),
            ("NULL", "NULL"),
            ("NULL", "NULL"),
        ];

        for (input, expected) in test_cases {
            let result = parse_bind_value(input)?;
            assert_eq!(
                result, expected,
                "For input '{}', expected '{}' but got '{}'",
                input, expected, result
            );
        }
        Ok(())
    }
}
