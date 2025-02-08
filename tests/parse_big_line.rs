use anyhow::Result;
use cubrid_logtopbind_rs::parser::*;

#[test]
fn test_parse_log_entries() -> Result<()> {
    let test_log = r#"[Q1]--------------------
21-02-24 15:30:45.123 (12345) execute srv_h_id 1 SELECT * FROM users WHERE id = $1
21-02-24 15:30:45.124 (12345) bind 1 : 42
21-02-24 15:30:45.125 (12345) execute 1 tuple 1 time 0.123
example.rs:123

[Q2]--------------------
21-02-24 15:30:46.123 (12345) execute_all srv_h_id 1 INSERT INTO users (name, age) VALUES ($1, $2)
21-02-24 15:30:46.124 (12345) bind 1 : John Doe
21-02-24 15:30:46.125 (12345) bind 2 : NULL
21-02-24 15:30:46.126 (12345) execute_all 1 tuple 1 time 0.234
test_file.rs:456

[Q3]--------------------
21-02-24 15:30:47.123 (12345) execute srv_h_id 1 UPDATE users SET name = $1, age = $2 WHERE id = $3
21-02-24 15:30:47.124 (12345) bind 1 : Jane Doe
with multiple
lines
21-02-24 15:30:47.125 (12345) bind 2 : 25
21-02-24 15:30:47.126 (12345) bind 3 : 42
21-02-24 15:30:47.127 (12345) execute 1 tuple 0 time 0.345
complex-query.rs:789"#;

    let entries = parse_log_entries(test_log)?;

    assert_eq!(entries.len(), 3);

    // Test first entry
    assert_eq!(entries[0].query_no, "1");
    assert_eq!(entries[0].query, "SELECT * FROM users WHERE id = $1");
    assert_eq!(entries[0].bind_statements, vec!["42"]);
    assert_eq!(entries[0].filename, "example.rs");

    // Test second entry
    assert_eq!(entries[1].query_no, "2");
    assert_eq!(
        entries[1].query,
        "INSERT INTO users (name, age) VALUES ($1, $2)"
    );
    assert_eq!(entries[1].bind_statements, vec!["John Doe", "NULL"]);
    assert_eq!(entries[1].filename, "test_file.rs");

    // Test third entry with multi-line query and bind statement
    assert_eq!(entries[2].query_no, "3");
    assert_eq!(
        entries[2].query,
        "UPDATE users SET name = $1, age = $2 WHERE id = $3"
    );
    assert_eq!(
        entries[2].bind_statements,
        vec!["Jane Doe\nwith multiple\nlines", "25", "42"]
    );
    assert_eq!(entries[2].filename, "complex-query.rs");

    Ok(())
}

#[test]
fn test_empty_input() -> Result<()> {
    let entries = parse_log_entries("")?;
    assert_eq!(entries.len(), 0);
    Ok(())
}

#[test]
fn test_partial_log_entry() -> Result<()> {
    let partial_log = r#"[Q1]--------------------
21-02-24 15:30:45.123 (12345) execute srv_h_id 1 SELECT * FROM users
21-02-24 15:30:45.124 (12345) bind 1 : 42"#;

    let entries = parse_log_entries(partial_log)?;

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].query_no, "1");
    assert_eq!(entries[0].query, "SELECT * FROM users");
    assert_eq!(entries[0].bind_statements, vec!["42"]);
    assert_eq!(entries[0].filename, "");

    Ok(())
}

#[test]
fn test_invalid_lines() -> Result<()> {
    let invalid_log = r#"[Q1]--------------------
21-02-24 15:30:45.123 (12345) execute srv_h_id 1 SELECT * FROM users
Invalid line that should be ignored
21-02-24 15:30:45.124 (12345) bind 1 : 42
Another invalid line
example.rs:123"#;

    let entries = parse_log_entries(invalid_log)?;

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].query_no, "1");
    assert_eq!(entries[0].query, "SELECT * FROM users");
    assert_eq!(entries[0].bind_statements, vec!["42"]);
    assert_eq!(entries[0].filename, "example.rs");

    Ok(())
}
