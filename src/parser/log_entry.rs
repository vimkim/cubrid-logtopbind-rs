use anyhow::Result;
use std::fmt;

#[derive(Default, Clone)]
pub struct LogEntry {
    pub query_no: String,
    pub filename: String,
    pub query: String,
    pub bind_statements: Vec<String>,
}

impl LogEntry {
    pub fn validate_entries(entries: &[LogEntry]) -> Result<()> {
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
        })
    }

    pub fn replace_query_params(query: &str, bind_statements: &[String]) -> Result<String> {
        let question_mark_count = query.chars().filter(|&c| c == '?').count();

        if question_mark_count != bind_statements.len() {
            return Err(anyhow::anyhow!(
                "Number of ? in query ({}) does not match number of bind parameters ({})",
                question_mark_count,
                bind_statements.len()
            ));
        }

        let mut result = query.to_string();
        for value in bind_statements {
            if let Some(pos) = result.find('?') {
                result.replace_range(pos..pos + 1, value);
            }
        }
        Ok(result)
    }
}

impl fmt::Debug for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "LogEntry {{")?;
        writeln!(f, "    query_no: {:?}", self.query_no)?;
        writeln!(f, "    filename: {:?}", self.filename)?;
        writeln!(f, "    query: {:?}", self.query)?;
        writeln!(f, "    bind_statements: [")?;
        writeln!(f, "        {}", self.bind_statements.join(",\n        "))?;
        writeln!(f, "    ]")?;
        write!(f, "}}")
    }
}
