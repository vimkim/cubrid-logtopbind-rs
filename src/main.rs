use anyhow::Result;
use cubrid_logtopbind_rs::{
    db::Database,
    parser::{parse_log_entries, LogEntry},
    utils::print_help,
};
use std::{env, fs};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let log_file = &args[1];
    let content = fs::read_to_string(log_file)?;

    println!("Parsing log entries...");
    let entries = parse_log_entries(&content)?;

    // debug_assert!(LogEntry::validate_entries(&entries).is_ok());
    // LogEntry::validate_entries(&entries)?; // instead of validate, I just filter out

    println!("Filtering out invalid entries...");
    let entries: Vec<LogEntry> = entries
        .into_iter()
        .filter(|entry| {
            let query_no_of_placeholders = entry.query.bytes().filter(|&b| b == b'?').count();
            query_no_of_placeholders == entry.bind_statements.len()
        })
        .collect();

    let mut db = Database::new("queries.db")?;
    db.initialize()?;

    println!("Processing log entries...");
    db.process_entries(&entries)?;

    Ok(())
}
