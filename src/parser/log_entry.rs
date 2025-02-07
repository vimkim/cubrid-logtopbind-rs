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
        // Split the query on '?' characters. For n placeholders, we expect n+1 parts.
        let parts: Vec<&str> = query.split('?').collect();

        // Validate that the number of '?' placeholders matches the number of bind parameters.
        if parts.len() - 1 != bind_statements.len() {
            return Err(anyhow::anyhow!(
                "Number of ? in query ({}) does not match number of bind parameters ({})",
                parts.len() - 1,
                bind_statements.len()
            ));
        }

        // Pre-calculate capacity to avoid multiple allocations.
        let additional_capacity: usize = bind_statements.iter().map(|s| s.len()).sum();
        let mut result = String::with_capacity(query.len() + additional_capacity);

        // Interleave each part with the corresponding bind parameter.
        // Note: parts.len() == bind_statements.len() + 1
        for (part, value) in parts.iter().zip(bind_statements.iter()) {
            result.push_str(part);
            result.push_str(value);
        }
        // Append the final part after the last placeholder.
        result.push_str(parts.last().unwrap());

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
