use clap::Parser;
use serde_json;
use std::path::PathBuf;
use std::process;
use thiserror::Error;

/// Enum defining the possible output formats for the report.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, Debug)]
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
    ParsingError(#[from] parser::parser::ParsingError),
    #[error("Failed to create simulation report: {0}")] // Placeholder for potential report errors
    ReportError(String), // Replace String with a specific error type if needed
}

fn main() -> Result<(), CliError> {
    let cli = Cli::parse();

    if !cli.log_file.exists() {
        eprintln!("Error: Log file not found: {}", cli.log_file.display());
        process::exit(1);
    }

    if let Err(e) = run(cli) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }

    Ok(())
}

fn run(cli: Cli) -> Result<(), CliError> {
    println!("Parsing log file: {}", cli.log_file.display());

    // Call the parser library function
    let events = parser::parser::parse_log_file(&cli.log_file)?;

    // Call the report generation function
    let report = parser::report::create_simulation_report(&events[..]);

    // Handle output based on the requested format
    match cli.output_format {
        OutputFormat::Summary => {
            eprintln!("Parsed {} events.", events.len());
            println!("\n{}\n", report);
        }
        OutputFormat::Json => {
            // Print as JSON (existing logic)
            let json_output = serde_json::to_string_pretty(&report)
                .map_err(|e| CliError::ReportError(format!("JSON serialization error: {}", e)))?;
            println!("{}", json_output);
        }
    }

    Ok(())
}
