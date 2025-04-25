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
 
Run the simulation script, specifying either `attrition` or `disk`:
```bash
# To run with Attritions.toml
./scripts/run_simulation.sh attrition

# To run with DiskFailureCycle.toml
./scripts/run_simulation.sh disk
```

This will generate trace logs in the `./events` directory.

## Parse traces

```bash
cargo run -- events/trace.0.0.0.0.69198.1745570893.x1jHWY.0.1.json
```

## Examples of report

### Attrition

```text
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

### Disk

```text
Parsing log file: events/trace.0.0.0.0.78823.1745574009.N18Xfo.0.1.json
Parsed 225 events.

FoundationDB Simulation Report
==============================
Seed: 3292690371
Simulated Time: 6m 30s 439ms
Real Time: 18s 539ms 700us

Cluster topology:
    Datacenter 0: 6 machines (sim_http_server: 1, storage_cache: 1, unset: 4)
    Datacenter 1: 5 machines (sim_http_server: 1, unset: 4)
    Datacenter 2: 7 machines (sim_http_server: 3, unset: 4)
    Datacenter 3: 6 machines (sim_http_server: 2, unset: 4)


--- Summaries ---
  Corrupted Blocks: 158
  Set Disk Failures: 5
```