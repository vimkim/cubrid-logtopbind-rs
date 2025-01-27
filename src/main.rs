use anyhow::Result;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use regex::Regex;
use rusqlite::{params, Connection};
use serde_json::Value;
use std::env;
use std::fmt;
use std::fs;

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
    let mut conn = Connection::open(db_path)?;
    initialize_db(&conn)?;

    // Read log file
    let content = fs::read_to_string(log_file).expect("Failed to read log file");

    // Parse log entries and insert them into the database
    let entries = parse_log_entries(&content);

    debug_assert!(validate_entries(&entries).is_ok(), "Validation failed");

    // Process entries with progress tracking
    insert_entry(&mut conn, &entries)?;

    rebind_queries(&mut conn, &entries)?;

    Ok(())
}

fn rebind_queries(conn: &mut Connection, entries: &[LogEntry]) -> Result<()> {
    // Connect to the database
    let conn = Connection::open("queries.db")?;

    // Query all entries
    let mut stmt =
        conn.prepare("SELECT query_no, filename, query, bind_statements FROM log_entries")?;

    let entries = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?, // query_no
            row.get::<_, String>(1)?, // filename
            row.get::<_, String>(2)?, // query
            row.get::<_, String>(3)?, // bind_statements
        ))
    })?;

    // Process each entry
    for entry in entries {
        let (query_no, filename, query, bind_statements) = entry?;

        match replace_query_params(&query, &bind_statements) {
            Ok(replaced_query) => {
                println!("Query No: {}", query_no);
                println!("Filename: {}", filename);
                println!("Original Query: {}", query);
                println!("Replaced Query: {}", replaced_query);
                println!("---");
            }
            Err(e) => {
                eprintln!("Error processing query {}: {}", query_no, e);
            }
        }
    }

    Ok(())
}

fn replace_query_params(
    query: &str,
    bind_statements: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Parse the JSON bind statements to get array of strings
    let bind_values: Value = serde_json::from_str(bind_statements)?;
    let bind_array = bind_values
        .as_array()
        .ok_or("bind_statements must be a JSON array")?;

    // Count the number of ? in the query
    let question_mark_count = query.chars().filter(|&c| c == '?').count();

    // Validate that we have the correct number of bind parameters
    if question_mark_count != bind_array.len() {
        return Err(format!(
            "Number of ? ({}) does not match number of bind parameters ({})",
            question_mark_count,
            bind_array.len()
        )
        .into());
    }

    // Replace each ? with corresponding string from bind_array
    let mut result = query.to_string();
    for value in bind_array {
        let str_value = value.as_str().ok_or("Bind parameter must be a string")?;

        // Replace first occurrence of ?
        if let Some(pos) = result.find('?') {
            result.replace_range(pos..pos + 1, str_value);
        }
    }

    Ok(result)
}

fn validate_entries(entries: &[LogEntry]) -> Result<()> {
    // number of '?' in entry.query and the number of bind statements should match
    entries.iter().try_for_each(|entry| {
        let query_no_of_placeholders = entry.query.matches('?').count();
        if query_no_of_placeholders != entry.bind_statements.len() {
            return Err(anyhow::anyhow!(
                "Number of placeholders in query {} and bind statements {} do not match for query_no: {}, {:#?}",
                query_no_of_placeholders,
                entry.bind_statements.len(),
                entry.query_no,
                entry
            ));
        }
        Ok(())
    })?;
    Ok(())
}

#[derive(Default)]
struct LogEntry {
    query_no: String,
    filename: String,
    query: String,
    bind_statements: Vec<String>,
}

impl fmt::Debug for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "LogEntry {{")?;
        writeln!(f, "    query_no: {:?}", self.query_no)?;
        writeln!(f, "    filename: {:?}", self.filename)?;
        writeln!(f, "    query: {:?}", self.query)?;
        writeln!(f, "    bind_statements: [")?;

        // print bine statement line by line
        writeln!(f, "        {}", self.bind_statements.join(",\n        "))?;

        writeln!(f, "    ]")?;
        write!(f, "}}")
    }
}

fn initialize_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS log_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            query_no TEXT NOT NULL,
            filename TEXT NOT NULL,
            query TEXT NOT NULL,
            bind_statements JSON NOT NULL
        )",
        [],
    )?;
    Ok(())
}

fn parse_log_entries(content: &str) -> Vec<LogEntry> {
    let mut entries = Vec::new();
    let re_query_no = Regex::new(r"^\[Q(\d+)\]").unwrap();
    let re_filename = Regex::new(r"^([\w\.]+):").unwrap();
    let re_query = Regex::new(r"(?:execute_all|execute) srv_h_id (.*)$").unwrap();
    let re_bind = Regex::new(r"bind \d+ : .+? (?:\(.*\))?(.*)$").unwrap();
    let re_bind_null = Regex::new(r"bind \d+ : NULL$").unwrap();
    let re_end = Regex::new(r"execute_all 0 tuple").unwrap();

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
                    bind_statements: bind_statements.clone(),
                });

                // Reset for the next entry
                bind_statements.clear();
            }

            current_query_no = caps[1].to_string();
        } else if let Some(caps) = re_filename.captures(line) {
            current_filename = caps[1].to_string();
        } else if let Some(caps) = re_query.captures(line) {
            current_query = caps[1].to_string();
        } else if let Some(caps) = re_bind.captures(line) {
            bind_statements.push(caps[1].to_string());
        } else if re_bind_null.captures(line).is_some() {
            bind_statements.push("NULL".to_owned());
        } else if re_end.captures(line).is_some() {
            /* end of query */
        } else if line.is_empty() {
            /* empty line */
        } else {
            panic!("Unrecognized line: {}", line);
        }
    }

    // Save the last entry
    if !current_query_no.is_empty() {
        entries.push(LogEntry {
            query_no: current_query_no,
            filename: current_filename,
            query: current_query,
            bind_statements,
        });
    }

    entries
}

fn insert_entry(conn: &mut Connection, entries: &[LogEntry]) -> Result<()> {
    let total_entries = entries.len();
    let progress_bar = ProgressBar::new(total_entries as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) - ETA: {eta}")
            .unwrap()
            .progress_chars("#>-"),
    );

    {
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO log_entries (query_no, filename, query, bind_statements) 
                VALUES (?1, ?2, ?3, ?4)",
            )?;

            for entry in entries {
                let bind_statements_json = serde_json::to_string(&entry.bind_statements)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                stmt.execute(params![
                    &entry.query_no,
                    &entry.filename,
                    &entry.query,
                    &bind_statements_json,
                ])?;
                progress_bar.inc(1);
            }
        } // stmt dropped here
        tx.commit()?;
    }

    progress_bar.finish_with_message("All log entries processed successfully!");
    Ok(())
}

fn print_help() {
    println!("Usage: program_name <log_file>");
    println!("\nArguments:");
    println!("  <log_file>        Path to the log file");
    println!("\nOptions:");
    println!("  -h, --help        Show this help message");
}
