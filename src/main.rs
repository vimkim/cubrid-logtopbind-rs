use regex::Regex;
use rusqlite::{params, Connection, Result};
use serde_json::json;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    // Define the log file from the first argument
    let log_file = &args[1];

    // Define the database file
    let db_path = "queries.db";

    // Initialize SQLite connection and table
    let conn = Connection::open(db_path)?;
    initialize_db(&conn)?;

    // Read log file
    let content = fs::read_to_string(log_file).expect("Failed to read log file");

    // Parse log entries and insert them into the database
    let entries = parse_log_entries(&content);
    for entry in entries {
        println!("Inserting log entry: {:?}", entry);
        insert_entry(&conn, &entry)?;
    }

    println!("Log entries have been saved to the database.");
    Ok(())
}

#[derive(Debug)]
struct LogEntry {
    query_no: String,
    filename: String,
    query: String,
    bind_statements: String, // JSON array as a string
}

fn initialize_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS log_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            query_no TEXT NOT NULL,
            filename TEXT NOT NULL,
            query TEXT NOT NULL,
            bind_statements TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

fn parse_log_entries(content: &str) -> Vec<LogEntry> {
    let mut entries = Vec::new();
    let re_query_no = Regex::new(r"^\[Q(\d+)]").unwrap();
    let re_filename = Regex::new(r"^([\w\.]+):").unwrap();
    let re_query = Regex::new(r"\d+ execute_all .*? ([A-Z]+.*?)\s+bind ").unwrap();
    let re_bind = Regex::new(r"bind \d+ : (.*?)\s*").unwrap();

    let mut current_query_no = "".to_string();
    let mut current_filename = "".to_string();
    let mut current_query = "".to_string();
    let mut bind_statements = Vec::new();

    for line in content.lines() {
        if let Some(caps) = re_query_no.captures(line) {
            if !current_query_no.is_empty() {
                // Save the previous entry
                entries.push(LogEntry {
                    query_no: current_query_no.clone(),
                    filename: current_filename.clone(),
                    query: current_query.clone(),
                    bind_statements: json!(bind_statements).to_string(),
                });

                // Reset for the next entry
                bind_statements.clear();
            }

            current_query_no = caps[1].to_string();
        }

        if let Some(caps) = re_filename.captures(line) {
            current_filename = caps[1].to_string();
        }

        if let Some(caps) = re_query.captures(line) {
            current_query = caps[1].to_string();
        }

        if let Some(caps) = re_bind.captures(line) {
            bind_statements.push(caps[1].to_string());
        }
    }

    // Save the last entry
    if !current_query_no.is_empty() {
        entries.push(LogEntry {
            query_no: current_query_no,
            filename: current_filename,
            query: current_query,
            bind_statements: json!(bind_statements).to_string(),
        });
    }

    entries
}

fn insert_entry(conn: &Connection, entry: &LogEntry) -> Result<()> {
    conn.execute(
        "INSERT INTO log_entries (query_no, filename, query, bind_statements) VALUES (?1, ?2, ?3, ?4)",
        params![
            entry.query_no,
            entry.filename,
            entry.query,
            entry.bind_statements
        ],
    )?;
    Ok(())
}

fn print_help() {
    println!("Usage: program_name <log_file>");
    println!("\nArguments:");
    println!("  <log_file>        Path to the log file");
    println!("\nOptions:");
    println!("  -h, --help        Show this help message");
}
