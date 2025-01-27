use anyhow::{Context, Result};
use clap::Parser;
use rusqlite::Connection;
use sqlformat::{FormatOptions, QueryParams};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Query number to look up
    #[arg(short, long)]
    query_no: String,
    /// Path to the SQLite database file
    #[arg(short, long, default_value = "queries.db")]
    database: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Connect to database
    let conn = Connection::open(&cli.database)
        .with_context(|| format!("Failed to open database: {}", cli.database))?;

    // Query the log entry
    let mut stmt = conn.prepare("SELECT replaced_query FROM logs WHERE query_no = ? LIMIT 1")?;
    let query: Option<String> = stmt.query_row([&cli.query_no], |row| row.get(0)).ok();

    match query {
        Some(sql) => {
            // Create formatting options
            let options = FormatOptions {
                indent: sqlformat::Indent::Spaces(4),
                lines_between_queries: 2,
                uppercase: None,
                ignore_case_convert: None,
            };

            // Since we don't have any parameters to bind, we'll use QueryParams::None
            // Note: The order matters! query -> params -> options
            let formatted = sqlformat::format(&sql, &QueryParams::None, &options);

            println!("Query #{}\n", cli.query_no);
            println!("{}", formatted);
        }
        None => {
            println!("No query found with number: {}", cli.query_no);
        }
    }
    Ok(())
}
