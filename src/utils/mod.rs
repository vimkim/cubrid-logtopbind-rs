pub mod query_format;
pub fn print_help() {
    println!("Usage: program_name <log_file>");
    println!("\nArguments:");
    println!("  <log_file>        Path to the log file");
    println!("\nOptions:");
    println!("  -h, --help        Show this help message");
}
