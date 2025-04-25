#!/usr/bin/env bash

# Run the FDB simulation with CombinedNemesis configuration
fdbserver -r simulation -f testfiles/DiskFailureCycle.toml --trace-format json -L ./events --logsize 1GiB
