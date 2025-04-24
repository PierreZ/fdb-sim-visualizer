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
            println!("\n--- Simulation Report (Summary) ---");
            println!("{}", report); // Use the Display impl for summary
        }
        OutputFormat::Json => {
            println!("\n--- Simulation Report (JSON) ---");
            // Attempt to serialize the report to JSON
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
