use clap::Parser;
use clap::ValueEnum;
use parser::parser::{parse_log_file, ParsingError};
use parser::report::create_simulation_report;
use serde_json;
use std::path::PathBuf;
use std::process;
use thiserror::Error;

/// Enum defining the possible output formats for the report.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum OutputFormat {
    /// Human-readable summary format (default)
    Summary,
    /// Full report in JSON format
    Json,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the FDB simulation log file (JSON format).
    #[arg(value_name = "FILE")]
    log_file: PathBuf,

    /// The desired output format for the simulation report.
    #[arg(long, value_enum, default_value_t = OutputFormat::Summary)]
    output_format: OutputFormat,
}

#[derive(Error, Debug)]
enum CliError {
    #[error("Failed to parse log file: {0}")]
    ParsingError(#[from] ParsingError),
    #[error("Failed to create simulation report: {0}")] // Placeholder for potential report errors
    ReportError(String), // Replace String with a specific error type if needed
}

fn main() {
    let cli = Cli::parse();

    if !cli.log_file.exists() {
        eprintln!("Error: Log file not found: {}", cli.log_file.display());
        process::exit(1);
    }

    if let Err(e) = run(cli) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), CliError> {
    println!("Parsing log file: {}", cli.log_file.display());

    // Call the parser library function
    let events = parse_log_file(&cli.log_file)?;

    // Call the report generation function
    let report = create_simulation_report(&events[..]); // Pass as slice reference

    // Handle output based on the requested format
    match cli.output_format {
        OutputFormat::Summary => {
            println!("Parsed {} events.", events.len());
            println!("--- Simulation Report (Summary) ---");
            println!("FoundationDB Simulation Report");
            println!("==============================");
            if let Some(seed) = &report.seed {
                println!("Seed: {}", seed);
            }
            if let Some(time) = &report.elapsed_time {
                println!("Elapsed Time: {} seconds", time);
            }
            println!(""); // Spacer

            // --- Machine Hierarchy --- //
            if !report.machine_details.is_empty() {
                println!("Cluster topology:");

                // Count machines per datacenter
                use std::collections::HashMap;
                let mut dc_counts: HashMap<String, usize> = HashMap::new();
                for machine in report.machine_details.values() {
                    let dc_id = machine.dc_id.as_deref().unwrap_or("N/A").to_string();
                    *dc_counts.entry(dc_id).or_insert(0) += 1;
                }

                let mut sorted_dcs: Vec<_> = dc_counts.keys().collect();
                sorted_dcs.sort(); // Sort by dc_id for consistent output
                for dc_id in sorted_dcs {
                    println!(
                        "    Datacenter {}: {} machines", // Updated per-DC format
                        dc_id, dc_counts[dc_id]
                    );
                }
                println!(); // Blank line after summary
            }

            println!("");

            println!("--- Summaries ---");
            if let Some(summary) = &report.clogging_pair_summary {
                println!("  Clogging Pairs:");
                println!("    Count: {}", summary.count);
                println!(
                    "    Duration (sec): Min={:.6}, Mean={:.6}, Max={:.6}",
                    summary.min_seconds, summary.mean_seconds, summary.max_seconds
                );
            }
            println!("  Clogged Interfaces (by Queue):");
            // Sort queue names for consistent output
            let mut queues: Vec<_> = report.clog_interface_summary.keys().collect();
            queues.sort();
            for queue_name in queues {
                if let Some(summary) = report.clog_interface_summary.get(queue_name) {
                    println!("    Queue '{}':", queue_name);
                    println!("      Count: {}", summary.count);
                    println!(
                        "      Delay (sec): Min={:.6}, Mean={:.6}, Max={:.6}",
                        summary.min_seconds, summary.mean_seconds, summary.max_seconds
                    );
                }
            }

            println!("  Assassinations (by KillType):");
            // Sort kill types for consistent output
            let mut kill_types: Vec<_> = report.assassination_summary.keys().collect();
            kill_types.sort();
            for kill_type in kill_types {
                if let Some(count) = report.assassination_summary.get(kill_type) {
                    println!("    {}: {}", kill_type, count);
                }
            }

            println!(
                "  Coordinator Changes: {}",
                report.coordinators_change_count
            );
            println!(""); // Spacer
        }
        OutputFormat::Json => {
            // Print as JSON (existing logic)
            match serde_json::to_string_pretty(&report) {
                Ok(json_report) => {
                    println!("{}", json_report);
                }
                Err(e) => {
                    eprintln!("Error serializing report to JSON: {}", e);
                    // Return an error or handle it as appropriate
                    // For now, just print the error and continue, maybe default to summary?
                    // Or define a proper error variant in CliError
                    return Err(CliError::ReportError(format!(
                        "JSON serialization failed: {}",
                        e
                    )));
                }
            }
        }
    }

    Ok(())
}
