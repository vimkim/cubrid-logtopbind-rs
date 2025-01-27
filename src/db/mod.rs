// In src/db/mod.rs

use crate::parser::LogEntry;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rusqlite::{params, Connection};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    pub fn initialize(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS log_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                query_no TEXT NOT NULL,
                filename TEXT NOT NULL,
                original_query TEXT NOT NULL,
                replaced_query TEXT,
                bind_statements JSON NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn process_entries(&mut self, entries: &[LogEntry]) -> Result<()> {
        let progress_bar = self.create_progress_bar(entries.len());

        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO log_entries (query_no, filename, original_query, replaced_query, bind_statements) 
                VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;

            for entry in entries {
                // Try to replace query parameters
                let replaced_query =
                    match LogEntry::replace_query_params(&entry.query, &entry.bind_statements) {
                        Ok(replaced) => {
                            println!("Successfully processed query {}:", entry.query_no);
                            println!("Filename: {}", entry.filename);
                            println!("Original: {}", entry.query);
                            println!("Replaced: {}", replaced);
                            println!("---");
                            replaced
                        }
                        Err(e) => {
                            eprintln!("Error processing query {}: {}", entry.query_no, e);
                            String::new() // Empty string for failed replacements
                        }
                    };

                // Convert bind statements to JSON
                let bind_statements_json = serde_json::to_string(&entry.bind_statements)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                // Insert into database
                stmt.execute(params![
                    &entry.query_no,
                    &entry.filename,
                    &entry.query,
                    &replaced_query,
                    &bind_statements_json,
                ])?;

                progress_bar.inc(1);
            }
        }
        tx.commit()?;

        progress_bar.finish_with_message("All log entries processed successfully!");
        Ok(())
    }

    fn create_progress_bar(&self, total_entries: usize) -> ProgressBar {
        let pb = ProgressBar::new(total_entries as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) - ETA: {eta}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        pb
    }
}
