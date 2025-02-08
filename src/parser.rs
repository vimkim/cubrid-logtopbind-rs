mod log_entry;

pub use log_entry::LogEntry;

use indicatif::{ProgressBar, ProgressStyle};

use anyhow::Result;
use parse_line::parse_bind_value;
use parse_line::parse_line;
use parse_line::ParsedLine;
use regex::Regex;

mod parse_line;

pub fn parse_log_entries(content: &str) -> Result<Vec<LogEntry>> {
    let timestamp_pattern =
        r"(?:\d{2})-(?:\d{2})-(?:\d{2})\s(?:\d{2}):(?:\d{2}):(?:\d{2})\.(?:\d{3})\s\((?:\d+)\)";

    let re_query_no = Regex::new(r"^\[Q(\d+)\]-+$").unwrap();
    let re_query = Regex::new(&format!(
        r"^{} (?:execute_all|execute) srv_h_id \d* (.*)$",
        timestamp_pattern
    ))
    .unwrap();
    let re_bind = Regex::new(&format!(r"^{} bind \d+ : ", timestamp_pattern)).unwrap();

    let re_end = Regex::new(&format!(
        r"^{} (?:execute_all|execute) (error:-)?\d+ tuple \d+ time .*$",
        timestamp_pattern
    ))
    .unwrap();
    let re_filename =
        Regex::new(r"^([a-zA-Z0-9][a-zA-Z0-9_\.-]{0,150}[a-zA-Z0-9]):\d{1,6}$").unwrap();

    let lines: Vec<&str> = content.lines().collect();
    let pb = ProgressBar::new(lines.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );

    let mut entries = Vec::new();
    let mut current = LogEntry::default();

    let mut after_bind = false;

    for line in lines {
        pb.inc(1);
        // First try to parse the line using regexes.
        match parse_line(
            line,
            &re_query_no,
            &re_bind,
            &re_query,
            &re_end,
            &re_filename,
        ) {
            Some(ParsedLine::QueryNo(text)) => {
                if !current.query_no.is_empty() {
                    entries.push(current.clone());
                    current = LogEntry::default();
                }
                current.query_no = text.to_string();
                // Reset the bind flag when starting a new query block.
                after_bind = false;
            }
            Some(ParsedLine::Bind(text)) => {
                let text = parse_bind_value(text)?;
                current.bind_statements.push(text);
                after_bind = true;
            }
            Some(ParsedLine::Query(text)) => {
                current.query = text.to_string();
                after_bind = false;
            }
            Some(ParsedLine::End) => {
                // You can use this branch to update state or finalize a block if needed.
                after_bind = false;
            }
            Some(ParsedLine::Filename(text)) => {
                current.filename = text.to_string();
                after_bind = false;
            }
            None if after_bind => {
                // If no regex matched and we're in an "after_bind" state,
                // treat this as a continuation of the last bind statement.
                if let Some(last) = current.bind_statements.last_mut() {
                    // Append the line to the previous bind statement.
                    last.reserve(line.len() + 1);
                    last.push('\n');
                    last.push_str(line);
                }
            }
            None if line.is_empty() => {
                // Ignore empty lines.
            }
            None => {
                // Log or handle unrecognized lines.
                println!("Unrecognized line: {}", line);
            }
        }
    }

    if !current.query_no.is_empty() {
        entries.push(current);
    }

    pb.finish_with_message("Parsing complete");
    Ok(entries)
}
