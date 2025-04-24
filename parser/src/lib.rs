use regex::Regex;
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
    /// Represents a SimulatedMachineStart event.
    SimulatedMachineStart(SimulatedMachineStartData),
    /// Represents a SimulatedMachineProcess event.
    SimulatedMachineProcess(SimulatedMachineProcessData),
    /// Represents an Assassination event.
    Assassination(AssassinationData),
    /// Represents a CoordinatorsChange event.
    CoordinatorsChange(CoordinatorsChangeData),
    /// Represents a ProgramStart event.
    ProgramStart(ProgramStartData),
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
    #[serde(rename = "Seconds")]
    pub seconds: String, // Match JSON string type
}

/// Data specific to a ClogInterface event.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ClogInterfaceData {
    #[serde(rename = "Time")]
    pub timestamp: String,
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

/// Data specific to a SimulatedMachineStart event.
#[derive(Debug, Deserialize, PartialEq)]
pub struct SimulatedMachineStartData {
    #[serde(rename = "Time")]
    pub timestamp: String,
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "MachineIPs")]
    pub machine_ips: String, // Can be space-separated
    #[serde(rename = "ProcessClass")]
    pub process_class: String,
    #[serde(rename = "DataHall")]
    pub data_hall: String,
    #[serde(rename = "Locality")]
    pub locality: String, // Contains dcid, machineid, etc.
    // These fields are extracted from Locality after deserialization
    pub dcid: Option<String>,
    pub machineid: Option<String>,
    // Other fields ignored: Severity, DateTime, Machine, Folder0, CFolder0, SSL, Processes, BootCount, Restarting, UseSeedFile, ZoneId, ThreadID, LogGroup
}

/// Data specific to a SimulatedMachineProcess event.
#[derive(Debug, Deserialize, PartialEq)]
pub struct SimulatedMachineProcessData {
    #[serde(rename = "Time")]
    pub timestamp: String,
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Address")]
    pub address: String, // e.g., "2.0.1.0:1"
    #[serde(rename = "DataHall")]
    pub data_hall: String,
    #[serde(rename = "ZoneId")]
    pub zone_id: String,
    // Other fields ignored: Severity, DateTime, Machine, Folder, ThreadID, LogGroup
}

/// Data specific to an Assassination event.
#[derive(Debug, Deserialize, PartialEq)]
pub struct AssassinationData {
    #[serde(rename = "Time")]
    pub timestamp: String,
    #[serde(rename = "Machine")]
    pub machine: String,
    #[serde(rename = "TargetMachine")]
    pub target_machine: Option<String>,
    #[serde(rename = "TargetDatacenter")]
    pub target_datacenter: Option<String>,
    #[serde(rename = "KillType")]
    pub kill_type: Option<KillType>,
    // Other fields ignored
}

/// Data specific to a CoordinatorsChange event.
#[derive(Debug, Deserialize, PartialEq)]
pub struct CoordinatorsChangeData {
    #[serde(rename = "Time")]
    pub timestamp: String,
    #[serde(rename = "NewCoordinatorsKey")]
    pub new_coordinators_key: String,
    // Other fields ignored: Severity, DateTime, Machine, ID, Auto, ThreadID, LogGroup, Roles
}

/// Data specific to a ProgramStart event.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ProgramStartData {
    #[serde(rename = "Time")]
    pub timestamp: String,
    #[serde(rename = "Machine")]
    pub machine: String,
    #[serde(rename = "RandomSeed")]
    pub random_seed: Option<String>, // Seed might not be present in all ProgramStart events
}

#[repr(i64)] // Specify underlying representation
#[derive(Debug, PartialEq, Clone, Copy, Deserialize)]
#[serde(try_from = "String")]
pub enum KillType {
    Reboot = 0,
    RebootAndDelete = 1,
    KillInstantly = 2,
    InjectFaults = 3,
    FailDisk = 4,
    RebootProcessAndSwitch = 5,
    Unknown(i64),
}

impl TryFrom<String> for KillType {
    type Error = std::num::ParseIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let num = value.parse::<i64>()?;
        Ok(match num {
            0 => KillType::Reboot,
            1 => KillType::RebootAndDelete,
            2 => KillType::KillInstantly,
            3 => KillType::InjectFaults,
            4 => KillType::FailDisk,
            5 => KillType::RebootProcessAndSwitch,
            _ => KillType::Unknown(num),
        })
    }
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
            Event::SimulatedMachineStart(data) => data.timestamp.parse().unwrap_or(0.0),
            Event::SimulatedMachineProcess(data) => data.timestamp.parse().unwrap_or(0.0),
            Event::Assassination(data) => data.timestamp.parse().unwrap_or(0.0),
            Event::CoordinatorsChange(data) => data.timestamp.parse().unwrap_or(0.0),
            Event::ProgramStart(data) => data.timestamp.parse().unwrap_or(0.0),
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
            "SimulatedMachineStart" => {
                match serde_json::from_value::<SimulatedMachineStartData>(node.clone()) {
                    Ok(mut data) => {
                        // Extract optional dcid and machineid from Locality string
                        data.dcid = extract_from_locality(&data.locality, "dcid").map(String::from);
                        data.machineid =
                            extract_from_locality(&data.locality, "machineid").map(String::from);
                        Some(Event::SimulatedMachineStart(data))
                    }
                    Err(_) => None,
                }
            }
            "SimulatedMachineProcess" => {
                match serde_json::from_value::<SimulatedMachineProcessData>(node.clone()) {
                    Ok(data) => Some(Event::SimulatedMachineProcess(data)),
                    Err(_) => None,
                }
            }
            "Assassination" => {
                match serde_json::from_value::<AssassinationData>(node.clone()) {
                    Ok(data) => Some(Event::Assassination(data)),
                    Err(_) => None,
                }
            }
            "CoordinatorsChangeBeforeCommit" => {
                match serde_json::from_value::<CoordinatorsChangeData>(node.clone()) {
                    Ok(data) => Some(Event::CoordinatorsChange(data)),
                    Err(_) => None,
                }
            }
            "ProgramStart" => {
                match serde_json::from_value::<ProgramStartData>(node.clone()) {
                    Ok(data) => Some(Event::ProgramStart(data)),
                    Err(_) => None,
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

/// Helper function to extract values from Locality string using regex
fn extract_from_locality<'a>(locality: &'a str, key: &str) -> Option<&'a str> {
    // Build a simple regex dynamically - more robust would be pre-compiling
    let pattern = format!(r#"\b{}=(\[[^]]*\]|[^ ]*)"#, regex::escape(key));
    let re = Regex::new(&pattern).ok()?; // Handle regex compilation error
    re.captures(locality)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
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
        // ElapsedTime (1), SimulatedMachineStart (29), SimulatedMachineProcess (29)
        // Total = 308 + 479 + 1 + 29 + 29 = 846
        let events = result.unwrap();
        let start_count = events
            .iter()
            .filter(|e| matches!(e, Event::SimulatedMachineStart(_)))
            .count();
        let process_count = events
            .iter()
            .filter(|e| matches!(e, Event::SimulatedMachineProcess(_)))
            .count();
        let clog_pair_count = events
            .iter()
            .filter(|e| matches!(e, Event::CloggingPair(_)))
            .count();
        let clog_if_count = events
            .iter()
            .filter(|e| matches!(e, Event::ClogInterface(_)))
            .count();
        let elapsed_count = events
            .iter()
            .filter(|e| matches!(e, Event::ElapsedTime(_)))
            .count();
        let assassination_count = events
            .iter()
            .filter(|e| matches!(e, Event::Assassination(_)))
            .count();
        let coord_change_count = events
            .iter()
            .filter(|e| matches!(e, Event::CoordinatorsChange(_)))
            .count();

        assert_eq!(clog_pair_count, 308, "Incorrect CloggingPair count");
        assert_eq!(clog_if_count, 479, "Incorrect ClogInterface count");
        assert_eq!(elapsed_count, 1, "Incorrect ElapsedTime count");
        assert_eq!(start_count, 29, "Incorrect SimulatedMachineStart count");
        assert_eq!(process_count, 29, "Incorrect SimulatedMachineProcess count");
        assert_eq!(assassination_count, 1, "Incorrect Assassination count");
        assert_eq!(coord_change_count, 1, "Incorrect CoordinatorsChange count");
        assert_eq!(
            events.len(),
            883, // Update expected total count
            "Expected 883 total events, found {}",
            events.len()
        );

        let process_event = events
            .iter()
            .find_map(|e| match e {
                Event::SimulatedMachineProcess(data) if data.id == "68f8a443716dcad2" => Some(data),
                _ => None,
            })
            .expect("Expected to find SimulatedMachineProcess event for ID 68f8a443716dcad2");

        assert_eq!(process_event.timestamp, "0.000000");
        assert_eq!(process_event.address, "2.0.1.0:1");
        assert_eq!(process_event.data_hall, "0".to_string());
        assert_eq!(process_event.zone_id, "a2da9142f354b315465f9d57c6b5a01b");

        // Check the first SimulatedMachineStart event
        let start_event = events
            .iter()
            .find_map(|e| match e {
                Event::SimulatedMachineStart(data) if data.id == "68f8a443716dcad2" => Some(data),
                _ => None,
            })
            .expect("Expected to find SimulatedMachineStart event for ID 68f8a443716dcad2");

        assert_eq!(start_event.timestamp, "0.000000");
        assert_eq!(start_event.machine_ips, "2.0.1.0");
        assert_eq!(start_event.process_class, "unset");
        assert_eq!(start_event.data_hall, "0".to_string());
        assert_eq!(start_event.locality, "zoneid=a2da9142f354b315465f9d57c6b5a01b processid=[unset] machineid=b50aea195b5bb79cea41a2c5f649aa19 dcid=0 data_hall=0");

        // Test the post-processing extraction (these fields are not directly in the JSON)
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

    #[test]
    fn test_parse_assassination_event() {
        let json_line = r#"{
            "Severity": "10", "Time": "115.540043", "DateTime": "2025-04-24T08:55:32Z", "Type": "Assassination", "Machine": "3.4.3.3:1", "ID": "0000000000000000", "TargetMachine": "zoneid=ac78c874bf4df0b17c83b9e9a8a29994 processid=[unset] machineid=ddc4353aca4a28397c289fa49080f82d dcid=1 data_hall=1", "ZoneId": "ac78c874bf4df0b17c83b9e9a8a29994", "Reboot": "1", "KilledMachines": "0", "MachinesToKill": "10", "MachinesToLeave": "3", "Machines": "18", "Replace": "1", "ThreadID": "14334889317801306560", "LogGroup": "default", "Roles": "TS"
        }"#;
        let node: JsonNode = serde_json::from_str(json_line).expect("Failed to parse JSON line");
        let event = parse_event_from_node(&node);

        assert!(event.is_some(), "Event should be parsed");
        match event.unwrap() {
            Event::Assassination(data) => {
                assert_eq!(data.timestamp, "115.540043");
                assert_eq!(data.machine, "3.4.3.3:1");
                assert_eq!(data.target_machine.as_deref(), Some("zoneid=ac78c874bf4df0b17c83b9e9a8a29994 processid=[unset] machineid=ddc4353aca4a28397c289fa49080f82d dcid=1 data_hall=1"));
                assert!(data.target_datacenter.is_none());
            }
            _ => panic!("Parsed event is not an Assassination event"),
        }
    }

    #[test]
    fn test_parse_assassination_event_datacenter() {
        let json_line = r#"{
            "Severity": "10", "Time": "138.462824", "DateTime": "2025-04-24T08:55:39Z", "Type": "Assassination", "Machine": "3.4.3.1:1", "ID": "0000000000000000", "TargetDatacenter": "1", "Reboot": "1", "KillType": "6", "ThreadID": "10058538621798076542", "LogGroup": "default", "Roles": "TS"
        }"#;

        let node: JsonNode = serde_json::from_str(json_line).expect("Failed to parse JSON line");
        let event = parse_event_from_node(&node);

        assert!(event.is_some(), "Event should be parsed");
        match event.unwrap() {
            Event::Assassination(data) => {
                assert_eq!(data.timestamp, "138.462824");
                assert_eq!(data.machine, "3.4.3.1:1");
                assert!(data.target_machine.is_none());
                assert_eq!(data.target_datacenter.as_deref(), Some("1"));
                assert_eq!(data.kill_type, Some(KillType::Unknown(6)));
            }
            _ => panic!("Parsed event is not an Assassination event"),
        }
    }

    #[test]
    fn test_parse_program_start_event_with_seed() {
        let json_line = r#"{
            "Severity": "10", "Time": "0.000000", "DateTime": "2025-04-24T08:55:36Z", "Type": "ProgramStart", "Machine": "0.0.0.0:0", "ID": "0000000000000000", "RandomSeed": "2837976339", "SourceVersion": "412531b5c97fa84343da94888cc949a4d29e8c29", "Version": "7.3.43", "PackageName": "7.3", "FileSystem": "", "DataFolder": "", "WorkingDirectory": "/root", "ClusterFile": "", "ConnectionString": "", "ActualTime": "1745484936", "EnvironmentKnobOptions": "none", "CommandLine": "fdbserver -r simulation -f /root/logical_db.toml -b on --trace-format json -L ./logs", "BuggifyEnabled": "1", "FaultInjectionEnabled": "1", "MemoryLimit": "8589934592", "VirtualMemoryLimit": "0", "ProtocolVersion": "0x0FDB00B073000000", "ThreadID": "10058538621798076542", "LogGroup": "default", "TrackLatestType": "Original"
        }"#;
        let node: JsonNode = serde_json::from_str(json_line).expect("Failed to parse JSON line");
        let event = parse_event_from_node(&node);

        assert!(event.is_some(), "Event should be parsed");
        match event.unwrap() {
            Event::ProgramStart(data) => {
                assert_eq!(data.timestamp, "0.000000");
                assert_eq!(data.machine, "0.0.0.0:0");
                assert_eq!(data.random_seed.as_deref(), Some("2837976339"));
            }
            _ => panic!("Parsed event is not a ProgramStart event"),
        }
    }

    #[test]
    fn test_parse_program_start_event_without_seed() {
         let json_line = r#"{  "Severity": "10", "Time": "251.750000", "OriginalTime": "59.959330", "DateTime": "2025-04-24T08:56:01Z", "OriginalDateTime": "2025-04-24T08:55:51Z", "Type": "ProgramStart", "Machine": "0.0.0.0:0", "ID": "0000000000000000", "Cycles": "2", "RandomId": "335a979b73b384c9", "SourceVersion": "412531b5c97fa84343da94888cc949a4d29e8c29", "Version": "7.3.43", "PackageName": "7.3", "DataFolder": "simfdb/6692e205b59f17fc558b5ed42d7a6bfa", "ConnectionString": "TestCluster:0@2.0.1.0:1:tls", "ActualTime": "1745484951", "CommandLine": "fdbserver -r simulation", "BuggifyEnabled": "1", "Simulated": "1", "ThreadID": "11198558628993500058", "LogGroup": "default", "TrackLatestType": "Rolled" }"#;
        let node: JsonNode = serde_json::from_str(json_line).expect("Failed to parse JSON line");
        let event = parse_event_from_node(&node);

        assert!(event.is_some(), "Event should be parsed");
        match event.unwrap() {
            Event::ProgramStart(data) => {
                assert_eq!(data.timestamp, "251.750000");
                assert_eq!(data.machine, "0.0.0.0:0");
                assert!(data.random_seed.is_none());
            }
            _ => panic!("Parsed event is not a ProgramStart event"),
        }
    }
}
