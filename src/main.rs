use anyhow::Result;
use cubrid_logtopbind_rs::{
    db::Database,
    parser::{parse_log_entries, LogEntry},
    utils::print_help,
};
use std::{
    env,
    fs::{self, File},
    io::BufWriter,
    io::Write,
};

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

    let entries = process_entries(entries)?;

    let mut db = Database::new("queries.db")?;
    db.initialize()?;

    println!("Processing log entries...");
    db.process_entries(&entries)?;

    Ok(())
}

fn process_entries(entries: Vec<LogEntry>) -> std::io::Result<Vec<LogEntry>> {
    // Partition the entries into valid and invalid groups.
    let (filtered_entries, deleted_entries): (Vec<LogEntry>, Vec<LogEntry>) =
        entries.into_iter().partition(|entry| {
            if entry.bind_statements.is_empty() {
                return true;
            }
            let placeholder_count = entry.query.bytes().filter(|&b| b == b'?').count();
            placeholder_count == entry.bind_statements.len()
        });

    // Print a debug log to the console for the problematic entries.
    println!("Deleted entries due to bind variable numbers mismatch:");
    for entry in &deleted_entries {
        println!("Entry number: {}", entry.query_no);
        println!("bind statements: {}", entry.bind_statements.len(),);
        println!(
            "placeholder_count: {}",
            entry.query.bytes().filter(|&b| b == b'?').count()
        );
        println!("Original query: {:.30}", entry.query);
        println!("-------------------------------------");
    }

    // Open the log file in append mode (creates it if it doesn't exist).
    // This ensures that previous log entries are preserved.
    let file = File::create("deleted_entries.log")?;
    let mut writer = BufWriter::new(file);

    // Write a detailed debug log to the file.
    for entry in deleted_entries {
        writeln!(writer, "{}", entry.query_no)?;
    }
    writer.flush()?;

    Ok(filtered_entries)
}
