use anyhow::{Context, Result};
use rusqlite::Connection;
use std::env;
use std::io::{self, Write};
use std::process;

fn execute_query(conn: &Connection, query: &str) -> Result<()> {
    let mut stmt = conn
        .prepare(query)
        .with_context(|| format!("Failed to prepare query: {}", query))?;

    let column_count = stmt.column_count();
    let column_names: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();

    // Only try to fetch rows for SELECT queries
    if query.trim().to_lowercase().starts_with("select") {
        let rows = stmt.query_map([], |row| {
            let mut values = Vec::new();
            for i in 0..column_count {
                let value: String = match row.get_ref(i)? {
                    rusqlite::types::ValueRef::Null => "NULL".to_string(),
                    rusqlite::types::ValueRef::Integer(i) => i.to_string(),
                    rusqlite::types::ValueRef::Real(f) => f.to_string(),
                    rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).to_string(),
                    rusqlite::types::ValueRef::Blob(b) => format!("<BLOB {}>", b.len()),
                };
                values.push(value);
            }
            Ok(values)
        })?;

        // Print column headers for SELECT queries
        if !column_names.is_empty() {
            println!("{}", column_names.join("|"));
            println!("{}", "-".repeat(column_names.join("|").len()));
        }

        // Print rows
        for row in rows {
            let row = row?;
            println!("{}", row.join("|"));
        }
    } else {
        // For non-SELECT queries, just execute and show affected rows
        let affected = stmt.execute([])?;
        println!("Query OK, {} row(s) affected", affected);
    }

    Ok(())
}

fn show_tables(conn: &Connection) -> Result<()> {
    let query = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;";
    let mut stmt = conn.prepare(query)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

    println!("Tables in database:");
    println!("------------------");
    for table_name in rows {
        println!("{}", table_name?);
    }
    println!();
    Ok(())
}

fn show_schema(conn: &Connection, table_name: Option<&str>) -> Result<()> {
    let query = match table_name {
        Some(name) => format!(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='{}';",
            name
        ),
        None => String::from("SELECT sql FROM sqlite_master WHERE type='table' ORDER BY name;"),
    };

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

    println!("Schema for tables:");
    println!("-----------------");
    for schema in rows {
        println!("{};", schema?);
        println!();
    }
    Ok(())
}

fn process_dot_command(conn: &Connection, command: &str) -> Result<bool> {
    match command.trim().to_lowercase().as_str() {
        ".tables" => {
            show_tables(conn)?;
            Ok(true)
        }
        ".schema" => {
            show_schema(conn, None)?;
            Ok(true)
        }
        ".quit" | ".exit" => Ok(false),
        cmd if cmd.starts_with(".schema ") => {
            let table_name = cmd.split_whitespace().nth(1).unwrap();
            show_schema(conn, Some(table_name))?;
            Ok(true)
        }
        _ => {
            println!("Unknown command: {}", command);
            println!("Available commands:");
            println!("  .tables             List tables");
            println!("  .schema [table]     Show schema for all tables or specific table");
            println!("  .quit or .exit      Exit the program");
            Ok(true)
        }
    }
}

fn interactive_mode(conn: &Connection) -> Result<()> {
    let mut buffer = String::new();

    loop {
        print!("sqlite> ");
        io::stdout().flush()?;

        buffer.clear();
        io::stdin().read_line(&mut buffer)?;

        let input = buffer.trim();

        // Skip empty lines
        if input.is_empty() {
            continue;
        }

        // Handle dot commands
        if input.starts_with('.') {
            if !process_dot_command(conn, input)? {
                break;
            }
            continue;
        }

        // Handle SQL queries
        if let Err(e) = execute_query(conn, input) {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  Interactive mode: sqlite-rs <database>");
        eprintln!("  Direct query:    sqlite-rs <database> <query>");
        process::exit(1);
    }

    let database = &args[1];
    let conn = Connection::open(database)
        .with_context(|| format!("Failed to open database: {}", database))?;

    if args.len() == 2 {
        // Interactive mode
        println!("SQLite Rust Shell version 0.1.0");
        println!("Enter \".help\" for usage hints.");
        println!("Connected to {}", database);
        interactive_mode(&conn)?;
    } else {
        // Direct query mode
        let query = &args[2];
        execute_query(&conn, query)?;
    }

    Ok(())
}
