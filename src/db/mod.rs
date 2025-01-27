// In src/db/mod.rs

use crate::parser::LogEntry;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rusqlite::{params, Connection};
use sqlformat::{FormatOptions, Indent, QueryParams};

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
            "CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                query_no TEXT NOT NULL,
                filename TEXT NOT NULL,
                original_query TEXT NOT NULL,
                binded_query TEXT,
                bind_vars JSON NOT NULL
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
                "INSERT INTO logs (query_no, filename, original_query, binded_query, bind_vars) 
                VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;

            for entry in entries {
                // Try to replace query parameters
                let replaced_query =
                    match LogEntry::replace_query_params(&entry.query, &entry.bind_statements) {
                        Ok(replaced) => replaced,
                        Err(e) => {
                            eprintln!("Error processing query {}: {}", entry.query_no, e);
                            String::new() // Empty string for failed replacements
                        }
                    };

                let options = FormatOptions {
                    indent: Indent::Spaces(4), // Use Indent enum instead of string
                    uppercase: Some(true),     // Option<bool> instead of bool
                    lines_between_queries: 1,
                    ignore_case_convert: None,
                };

                // format sql to be human readable, using sqlformat
                let formatted_query =
                    sqlformat::format(&replaced_query, &QueryParams::None, &options);

                // Convert bind statements to JSON
                let bind_statements_json = serde_json::to_string(&entry.bind_statements)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                // Insert into database
                stmt.execute(params![
                    &entry.query_no,
                    &entry.filename,
                    &entry.query,
                    &formatted_query,
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
