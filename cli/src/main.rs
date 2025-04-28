//! Command-line interface for the FDB Simulation Visualizer.

use clap::Parser as ClapParser; // Alias clap's Parser
use parser::{parser::parse_log_file, report::create_simulation_report}; // Use items from the parser library crate
use std::{error::Error, path::PathBuf}; // Import std::process
use thiserror::Error;

/// Enum defining the possible output formats for the report.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, Debug)]
enum OutputFormat {
    /// Human-readable summary format (default)
    Summary,
    /// Full report in JSON format
    Json,
}

#[derive(ClapParser, Debug)]
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
}

// Declare the tui module
mod tui;

/// Command line arguments
#[derive(ClapParser, Debug)]
#[command(author, version, about = "A TUI for visualizing FDB simulation logs.", long_about = None)]
struct Args {
    /// Path to the FDB simulation JSON log file
    #[arg(short, long)]
    log_file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Parse the log file and create the report using the parser crate
    println!("Parsing log file: {}", args.log_file.display());
    let events = parse_log_file(&args.log_file).expect("Error parsing log file");
    println!("Parsed {} events.", events.len());

    // Create the simulation report
    println!("Generating simulation report...");
    let report = create_simulation_report(&events);
    println!("Report generated.");

    // Setup terminal
    let mut terminal =
        tui::setup_terminal().map_err(|e| format!("Failed to setup terminal: {}", e))?;

    // Create app and run it
    let mut app = tui::App::new(report); // Pass the report to the TUI app
    let run_result = app.run(&mut terminal);

    // Restore terminal even if the app run fails
    tui::restore_terminal(&mut terminal)
        .map_err(|e| format!("Failed to restore terminal: {}", e))?;

    // Handle potential error from app run
    if let Err(err) = run_result {
        eprintln!("Error running TUI: {:?}", err);
        return Err(format!("TUI application error: {}", err).into());
    }

    Ok(())
}
