# CUBRID Log Analysis Tool

A high-performance tool for analyzing CUBRID broker logs, converting them into human-readable format and storing them in SQLite for easy querying and analysis.

## Overview

This tool helps CUBRID engineers and users inspect and analyze broker logs by:

- Converting raw CUBRID broker log files into a SQLite database
- Rebinding variables into their original SQL queries
- Formatting SQL queries for better readability
- Providing an interactive SQL interface for log analysis

## Prerequisites

- Rust (latest stable version)
- SQLite 3 (optional)
- Just command runner (optional)

## Installation

1. Clone the repository:

```bash
git clone [your-repository-url]
cd cubrid-logtopbind-rs
```

2. Build the project:

```bash
cargo build --release
```

Or using Just:

```bash
just release
```

## Usage

### Basic Usage

Convert a broker log file to SQLite database:

```bash
./target/release/logtopbind path/to/your/broker.log
```

This will create a `queries.db` file in your current directory.

### Query Inspection Utility

The `logtopprint` utility allows you to quickly inspect specific queries by their query number:

```bash
cargo build --bin logtopprint
./target/debug/logtopprint --query-no <QUERY_NO>
```

Available options:

```
Options:
  -q, --query-no <QUERY_NO>  Query number to look up
  -d, --database <DATABASE>  Path to the SQLite database file [default: queries.db]
  -h, --help                 Print help
  -V, --version              Print version
```

Example usage:

```bash
./target/debug/logtopprint --query-no 3
```

### Interactive SQL Query Mode

To analyze the converted logs using SQL:

```bash
./target/release/sqlite-rs queries.db
```

Or run specific queries:

```bash
./target/release/sqlite-rs queries.db 'SELECT * FROM logs;'
```

### Database Schema

The tool creates a SQLite database with the following schema:

```sql
CREATE TABLE IF NOT EXISTS logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query_no TEXT NOT NULL,
    filename TEXT NOT NULL,
    original_query TEXT NOT NULL,
    replaced_query TEXT,
    bind_vars JSON NOT NULL
);
```

### Common Query Examples

1. View all queries with their bound variables:

```sql
SELECT query_no, replaced_query FROM logs;
```

2. Get the first bind variable from each query:

```sql
SELECT bind_vars -> '$[0]' FROM logs;
```

## Development

### Available Commands

The project uses Just as a command runner. Here are some useful commands:

```bash
# Build the project
just build
# Run tests
just test
# Format code
just format
# Lint code
just lint
# Run with test data
just run-logtopbind-simple
# Run with larger dataset (500k entries)
just run-logtopbind-500k
```

### Performance Testing

The tool includes performance test targets:

```bash
# Test with small dataset
just run-logtopbind-simple
# Test with large dataset (500k entries)
just run-logtopbind-500k
```

### Dependencies

- `anyhow`: Error handling
- `indicatif`: Progress bars
- `regex`: Regular expression parsing
- `rusqlite`: SQLite database interface
- `serde_json`: JSON processing
- `sqlformat`: SQL formatting

## License

[BSD]
