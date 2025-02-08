#!/bin/bash

# Check if a filename is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <filename>"
    exit 1
fi

# Verify the file exists
if [ ! -f "$1" ]; then
    echo "File $1 not found!"
    exit 1
fi

# Use awk to remove everything up to "bind <number> : " and then print the rest.
awk '{ sub(/.*bind [0-9][0-9]* *: */, ""); print }' "$1"
