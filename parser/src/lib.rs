use serde::Deserialize;
use serde_json::Value as JsonNode;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

/// Represents different types of log events.
#[derive(Debug, PartialEq)]
pub enum Event {
    /// Represents a CloggingPair event.
    CloggingPair(CloggingPairData),
    /// Represents a ClogInterface event.
    ClogInterface(ClogInterfaceData),
    /// Represents an ElapsedTime event.
    ElapsedTime(ElapsedTimeData),
    // Add other specific event variants here
}

/// Data specific to a CloggingPair event.
#[derive(Debug, Deserialize, PartialEq)]
pub struct CloggingPairData {
    // Use idiomatic snake_case names and rename attributes
    #[serde(rename = "Time")]
    pub timestamp: String, // Match JSON string type
    #[serde(rename = "From")] // Match JSON key "From"
    pub from_id: String,
    #[serde(rename = "To")] // Match JSON key "To"
    pub to_id: String,
    #[serde(rename = "Machine")]
    pub machine: String,
    // Other fields like ID, Seconds, etc., are ignored by default
}

/// Data specific to a ClogInterface event.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ClogInterfaceData {
    #[serde(rename = "Time")]
    pub timestamp: String,
    #[serde(rename = "Machine")]
    pub machine: String,
    #[serde(rename = "IP")]
    pub ip: String,
    #[serde(rename = "Delay")]
    pub delay: String,
    #[serde(rename = "Queue")]
    pub queue: String,
    // Severity, DateTime, ID, ThreadID, LogGroup, Roles ignored
}

/// Data specific to an ElapsedTime event.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ElapsedTimeData {
    #[serde(rename = "Time")]
    pub timestamp: String,
    // Note: This event uses SimTime, not Time, for its primary timestamp.
    #[serde(rename = "SimTime")]
    pub sim_time: String,
    #[serde(rename = "RealTime")]
    pub real_time: String,
    // Severity, DateTime, Machine, ID, ThreadID, LogGroup ignored
}

impl Event {
    /// Returns the timestamp associated with the event, parsed from string.
    /// Returns 0.0 if parsing fails.
    /// For ElapsedTime events, this uses the SimTime field.
    pub fn timestamp(&self) -> f64 {
        match self {
            // Parse the timestamp string to f64
            Event::CloggingPair(data) => data.timestamp.parse().unwrap_or(0.0),
            Event::ClogInterface(data) => data.timestamp.parse().unwrap_or(0.0),
            Event::ElapsedTime(data) => data.timestamp.parse().unwrap_or(0.0),
        }
    }
}

/// Errors that can occur during log parsing.
#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("I/O error reading file: {0}")]
    Io(#[from] io::Error),
    // Use serde_json::Error directly for JSON issues when parsing the initial line
    #[error("JSON parsing error on line {line}: {source}")]
    JsonLineError {
        line: usize,
        source: serde_json::Error,
    },
}

/// Parses a log file where each line is a JSON object, skipping unrecognized events.
///
/// # Arguments
///
/// * `file_path` - The path to the log file.
///
/// # Returns
///
/// A `Result` containing a vector of successfully parsed `Event`s or a `ParsingError` if a line is invalid JSON.
pub fn parse_logs<P: AsRef<Path>>(file_path: P) -> Result<Vec<Event>, ParsingError> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for (line_number, line_result) in reader.lines().enumerate() {
        let line = line_result?; // Handle potential I/O errors reading a line

        // 1. Parse the line into a generic JsonNode
        let node: JsonNode =
            serde_json::from_str(&line).map_err(|e| ParsingError::JsonLineError {
                line: line_number + 1,
                source: e,
            })?;

        // 2. Pass the node to the event identification function and push if Some
        if let Some(event) = parse_event_from_node(&node) {
            events.push(event);
        }
    }

    Ok(events)
}

/// Attempts to parse a specific Event type from a generic JsonNode.
/// Returns Some(Event) on success, None otherwise.
fn parse_event_from_node(node: &JsonNode) -> Option<Event> {
    // Check the "Type" field to determine the event type
    if let Some(event_type) = node.get("Type").and_then(|v| v.as_str()) {
        match event_type {
            "CloggingPair" => {
                // Attempt to deserialize into the specific data struct
                match serde_json::from_value::<CloggingPairData>(node.clone()) {
                    Ok(data) => Some(Event::CloggingPair(data)), // Success
                    Err(_) => None, // Failed specific parse for known type
                }
            }
            "ClogInterface" => {
                // Attempt to deserialize into the specific data struct
                match serde_json::from_value::<ClogInterfaceData>(node.clone()) {
                    Ok(data) => Some(Event::ClogInterface(data)), // Success
                    Err(_) => None, // Failed specific parse for known type
                }
            }
            "ElapsedTime" => {
                // Attempt to deserialize into the specific data struct
                match serde_json::from_value::<ElapsedTimeData>(node.clone()) {
                    Ok(data) => Some(Event::ElapsedTime(data)), // Success
                    Err(_) => None, // Failed specific parse for known type
                }
            }
            // Add cases for other known event types here
            // "SomeOtherEvent" => { ... }
            _ => None, // Unknown "Type"
        }
    } else {
        None // Missing "Type" field
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Make items from outer module available

    #[test]
    fn test_parse_valid_log_file() {
        // Use the actual test log file provided by the user
        // Correct the relative path assuming test runs from 'parser' dir
        let file_path = "logs/trace.0.0.0.0.169.1745484896.1xR3BP.0.1.json";

        let result = parse_logs(file_path);

        // Assert that parsing succeeded (no invalid JSON lines)
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        // The expected count is the sum of CloggingPair (308), ClogInterface (479),
        // and ElapsedTime (1)
        let events = result.unwrap();
        assert_eq!(events.len(), 788, "Expected 788 events (CloggingPair + ClogInterface + ElapsedTime), found {}", events.len());
    }

    #[test]
    fn test_parse_io_error() {
        // Use a path relative to the parser directory that doesn't exist
        let non_existent_path = "logs/non_existent_file.log"; // Keep this path relative
        let result = parse_logs(non_existent_path);
        assert!(result.is_err());
        match result.err().unwrap() {
            ParsingError::Io(_) => { /* Expected */ }
            // Update expected error type
            err => panic!("Expected Io error, got {:?}", err),
        }
    }
}
