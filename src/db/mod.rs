use crate::parser::LogEntry;
use crate::utils::query_format::adhoc_fix_query;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rusqlite::{params, Connection};
use sqlformat::{FormatOptions, Indent};

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
                replaced_query TEXT,
                bind_vars JSON NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn process_entries(&mut self, entries: &[LogEntry]) -> Result<()> {
        let progress_bar = self.create_progress_bar(entries.len());
        let mut prepared_entries = Vec::with_capacity(entries.len());

        let _options = FormatOptions {
            indent: Indent::Spaces(4), // Use Indent enum instead of string
            uppercase: Some(true),     // Option<bool> instead of bool
            lines_between_queries: 1,
            ignore_case_convert: None,
        };

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

            let fixed_query = adhoc_fix_query(&replaced_query);

            // Convert bind statements to JSON
            let bind_statements_json = serde_json::to_string(&entry.bind_statements)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            prepared_entries.push((
                entry.query_no.clone(),
                entry.filename.clone(),
                entry.query.clone(),
                fixed_query,
                bind_statements_json,
            ));
            progress_bar.inc(1);
        }
        progress_bar.finish_with_message("All log entries processed successfully!");

        println!("Inserting log entries into database...");
        let progress_bar = self.create_progress_bar(entries.len());
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO logs (query_no, filename, original_query, replaced_query, bind_vars) 
                VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;

            for (query_no, filename, query, formatted_query, bind_statements_json) in
                prepared_entries
            {
                // Insert into database
                stmt.execute(params![
                    &query_no,
                    &filename,
                    &query,
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
