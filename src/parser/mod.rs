mod log_entry;
pub use log_entry::LogEntry;

use indicatif::{ProgressBar, ProgressStyle};

use anyhow::Result;

pub fn parse_log_entries(content: &str) -> Result<Vec<LogEntry>> {
    use regex::Regex;
    let re_query_no = Regex::new(r"^\[Q(\d+)\]").unwrap();
    let re_filename = Regex::new(r"^([\w\.]+):").unwrap();
    let re_query = Regex::new(r"(?:execute_all|execute) srv_h_id (?:\d+)? (.*)$").unwrap();
    let re_bind = Regex::new(r"bind \d+ : .+? (?:\(.*\))?(.*)$").unwrap();
    let re_bind_null = Regex::new(r"bind \d+ : NULL$").unwrap();
    let re_end = Regex::new(r"(?:execute_all|execute) .* tuple").unwrap();

    let lines: Vec<&str> = content.lines().collect();
    let pb = ProgressBar::new(lines.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )?
            .progress_chars("#>-"),
    );

    let mut entries = Vec::new();
    let mut current = LogEntry::default();

    for line in lines {
        pb.inc(1);

        if let Some(caps) = re_query_no.captures(line) {
            if !current.query_no.is_empty() {
                entries.push(current.clone());
                current = LogEntry::default();
            }
            current.query_no = caps[1].to_string();
        } else if let Some(caps) = re_filename.captures(line) {
            current.filename = caps[1].to_string();
        } else if let Some(caps) = re_query.captures(line) {
            current.query = caps[1].to_string();
        } else if let Some(caps) = re_bind.captures(line) {
            current.bind_statements.push(caps[1].to_string());
        } else if re_bind_null.captures(line).is_some() {
            current.bind_statements.push("NULL".to_owned());
        } else if re_end.captures(line).is_some() || line.is_empty() {
            continue;
        } else {
            // print error
            println!("Unrecognized line: {}", line);
        }
    }

    if !current.query_no.is_empty() {
        entries.push(current);
    }

    pb.finish_with_message("Parsing complete");
    Ok(entries)
}
