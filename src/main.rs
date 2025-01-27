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

    let entries = parse_log_entries(&content);
    debug_assert!(
        LogEntry::validate_entries(&entries).is_ok(),
        "Validation failed"
    );

    let mut db = Database::new("queries.db")?;
    db.initialize()?;
    db.insert_entries(&entries)?;
    db.rebind_queries(&entries)?;

    Ok(())
}
