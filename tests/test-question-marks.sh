#!/bin/bash
# Exit immediately if a command exits with a non-zero status,
# treat unset variables as an error, and ensure pipelines return the
# exit status of the last command to exit with a non-zero status.
set -euo pipefail

# Run the command and capture its output.
# Note: Adjust the path to your executable if needed.
output=$(./target/debug/sqlite-rs queries.db 'select replaced_query from logs')

# Check if the command ran successfully.
# The "set -e" above would normally exit the script on a non-zero status,
# but if you want a custom error message, you can do an explicit check:
if [ $? -ne 0 ]; then
    echo "Error: The command failed to execute."
    exit 1
fi

# Debug: Uncomment the following line to see the output when testing.
# echo "Command output: $output"

# Check if the output contains any '?' characters.
# The grep -F option treats the pattern as a fixed string.
if echo "$output" | grep -Fq '?'; then
    echo "Test failed: Output contains one or more question marks ('?')."
    exit 1
else
    echo "Test passed: Output does not contain any question marks."
fi

exit 0
