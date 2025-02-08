#!/bin/bash
set -euo pipefail

# Description:
# This script runs the 'logtopbind' binary in release mode, compares the generated
# log file 'deleted_entries.log' against the expected output in 'testdata/deleted_entries.txt',
# and cleans up the log file after the test.

# Run the binary with the provided test data.
echo "Running cargo command..."
cargo run --release --bin logtopbind ./testdata/log_top_50m.q

# Verify that the generated log file exists.
if [ ! -f "deleted_entries.log" ]; then
    echo "Error: 'deleted_entries.log' was not created. Exiting."
    exit 1
fi

# Compare the generated log file to the expected output.
echo "Comparing deleted_entries.log with ./testdata/deleted_entries.txt..."
if diff -u ./testdata/deleted_entries.txt deleted_entries.log; then
    echo "Test passed: Files match."
else
    echo "Test failed: Files differ."
    exit 1
fi

# Clean up the generated file.
echo "Cleaning up..."
rm -f deleted_entries.log

echo "Test completed successfully."
