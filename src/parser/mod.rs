mod log_entry;
use std::io::{self, Write};

pub use log_entry::LogEntry;

use indicatif::{ProgressBar, ProgressStyle};

use anyhow::Result;
use regex::Regex;

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

    let re_bind_null = Regex::new(&format!(r"^{} bind \d+ : NULL$", timestamp_pattern)).unwrap();
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
        // println!("{}", i);
        let mut after_bind_continue = false;
        pb.inc(1);
        io::stdout().flush().unwrap();

        if let Some(caps) = re_query_no.captures(line) {
            if !current.query_no.is_empty() {
                entries.push(current.clone());
                current = LogEntry::default();
            }
            current.query_no = caps[1].to_string();
        } else if re_bind_null.captures(line).is_some() {
            current.bind_statements.push("NULL".to_owned());
        } else if let Some(mat) = re_bind.find(line) {
            // mat.end() gives the index right after the matched prefix.
            // Everything after this position is the text you want.
            let captured_text = &line[mat.end()..];
            current.bind_statements.push(captured_text.to_string());
        } else if let Some(caps) = re_query.captures(line) {
            current.query = caps[1].to_string();
        } else if re_end.captures(line).is_some() {
        } else if let Some(caps) = re_filename.captures(line) {
            current.filename = caps[1].to_string();
        } else if after_bind {
            after_bind_continue = true;
            // append to the last element of the bind_statements
            let last = current.bind_statements.len() - 1;
            current.bind_statements[last] = format!("{}\n{}", current.bind_statements[last], line);
        } else if line.is_empty() {
            // ignore empty lines
            // must be after 'after_bind' check, since bind vars can contain empty lines
        } else {
            println!("Unrecognized line: {}", line);
        }

        after_bind = after_bind_continue || re_bind.captures(line).is_some();
    }

    if !current.query_no.is_empty() {
        entries.push(current);
    }

    pb.finish_with_message("Parsing complete");
    Ok(entries)
}
