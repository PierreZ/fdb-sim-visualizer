# fdb-sim-visualizer

## Requirements

- Rust
- fdbserver (to generate traces)

If you are using nix, everything is already setup for you within the flake.

## Build it

```bash
cargo build --release
```

## Generate traces

Run the simulation using the provided script:

```bash
./scripts/run_simulation.sh
```

This will generate trace logs in the `./events` directory.

## Parse traces

```bash
cargo run -- events/trace.0.0.0.0.69198.1745570893.x1jHWY.0.1.json
```

## Example of report

```text
❯ cargo run -- events/trace.0.0.0.0.69198.1745570893.x1jHWY.0.1.json
Parsing log file: events/trace.0.0.0.0.69198.1745570893.x1jHWY.0.1.json
Parsed 1025 events.

FoundationDB Simulation Report
==============================
Seed: 2993461222
Simulated Time: 170.719 seconds
Real Time: 18.0447 seconds

Cluster topology:
    Datacenter 0: 9 machines (sim_http_server: 1, storage: 4, storage_cache: 1, unset: 3)


--- Summaries ---
  Clogging Pairs:
    Count: 270
    Duration (sec): Min=0.000666, Mean=0.688360, Max=5.471810
  Clogged Interfaces (by Queue):
    Queue 'All':
      Count: 264
      Delay (sec): Min=0.000194, Mean=0.365979, Max=3.993960
    Queue 'Receive':
      Count: 207
      Delay (sec): Min=0.000052, Mean=0.353460, Max=4.732520
    Queue 'Send':
      Count: 198
      Delay (sec): Min=0.000057, Mean=0.317103, Max=4.606360
  Assassinations (by KillType):
    RebootAndDelete: 6
  Coordinator Changes: 3
```