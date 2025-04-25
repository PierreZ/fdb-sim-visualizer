#!/usr/bin/env bash

# Check if an argument was provided
if [ -z "$1" ]; then
  echo "Usage: $0 <attrition>"
  exit 1
fi

# Determine the configuration file based on the argument
TEST_FILE=""
case "$1" in
  attrition)
    TEST_FILE="testfiles/Attritions.toml"
    ;;
  *)
    echo "Invalid argument: $1. Use 'attrition'."
    exit 1
    ;;
esac

# Check if the file exists
if [ ! -f "$TEST_FILE" ]; then
  echo "Error: Configuration file not found: $TEST_FILE"
  exit 1
fi

echo "Running simulation with configuration: $TEST_FILE"

fdbserver -r simulation -f "$TEST_FILE" --trace-format json -L ./events --logsize 1GiB
