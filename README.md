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
:w
