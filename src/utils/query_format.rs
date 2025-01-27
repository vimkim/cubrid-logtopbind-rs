use regex::Regex;

pub fn adhoc_fix_query(query: &str) -> String {
    // Create regex pattern that matches SQL comments followed by either tabs or spaces
    let pattern = Regex::new(r"--.*?(?:\t{2,}|\s{8,})").unwrap();

    // Replace matched patterns by adding a newline
    pattern
        .replace_all(query, |caps: &regex::Captures| format!("{}\n", &caps[0]))
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_with_tabs() {
        let input = "SELECT * FROM users --comment here\t\t";
        let expected = "SELECT * FROM users --comment here\t\t\n";
        assert_eq!(adhoc_fix_query(input), expected);
    }

    #[test]
    fn test_comment_with_spaces() {
        let input = "SELECT * FROM users --comment here        ";
        let expected = "SELECT * FROM users --comment here        \n";
        assert_eq!(adhoc_fix_query(input), expected);
    }

    #[test]
    fn test_multiple_comments() {
        let input = "SELECT * FROM users --comment1\t\t\nWHERE id = 1 --comment2        ";
        let expected = "SELECT * FROM users --comment1\t\t\n\nWHERE id = 1 --comment2        \n";
        assert_eq!(adhoc_fix_query(input), expected);
    }

    #[test]
    fn test_comment_with_insufficient_whitespace() {
        let input = "SELECT * FROM users --comment here\t"; // Only one tab
        let expected = "SELECT * FROM users --comment here\t"; // No change
        assert_eq!(adhoc_fix_query(input), expected);

        let input2 = "SELECT * FROM users --comment here       "; // Only 7 spaces
        let expected2 = "SELECT * FROM users --comment here       "; // No change
        assert_eq!(adhoc_fix_query(input2), expected2);
    }
}
