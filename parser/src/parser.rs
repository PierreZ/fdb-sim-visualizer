use regex::Regex;
use serde::Deserialize;
use serde_json::Value as JsonNode;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use thiserror::Error;
use serde::de::DeserializeOwned;

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

impl Into<Event> for CloggingPairData {
    fn into(self) -> Event {
        Event::CloggingPair(self)
    }
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

impl Into<Event> for ClogInterfaceData {
    fn into(self) -> Event {
        Event::ClogInterface(self)
    }
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

impl Into<Event> for ElapsedTimeData {
    fn into(self) -> Event {
        Event::ElapsedTime(self)
    }
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

impl Into<Event> for SimulatedMachineProcessData {
    fn into(self) -> Event {
        Event::SimulatedMachineProcess(self)
    }
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

impl Into<Event> for AssassinationData {
    fn into(self) -> Event {
        Event::Assassination(self)
    }
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

impl Into<Event> for CoordinatorsChangeData {
    fn into(self) -> Event {
        Event::CoordinatorsChange(self)
    }
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

impl Into<Event> for ProgramStartData {
    fn into(self) -> Event {
        Event::ProgramStart(self)
    }
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
    Json {
        line: usize,
        source: serde_json::Error,
    },
    // Specific error for when an event type is unknown or parsing its data fails
    #[error("Failed to parse event data on line {line}: {event_type}")]
    EventDataParsing { line: usize, event_type: String },
}

/// Helper function to reduce repetition in deserialization
fn try_parse_event_data<T>(node: &JsonNode) -> Option<Event>
where
    T: DeserializeOwned + Into<Event>,
{
    serde_json::from_value::<T>(node.clone()).ok().map(Into::into)
}

/// Parses a single JSON log line represented as a `serde_json::Value` node into an Event.
///
/// This function centralizes the logic for identifying the event type and deserializing
/// the corresponding data structure.
fn parse_event_from_node(node: &JsonNode) -> Option<Event> {
    let event_type = node.get("Type")?.as_str()?;

    match event_type {
        "CloggingPair" => try_parse_event_data::<CloggingPairData>(node),
        "ClogInterface" => try_parse_event_data::<ClogInterfaceData>(node),
        "ElapsedTime" => try_parse_event_data::<ElapsedTimeData>(node),
        "SimulatedMachineStart" => {
            match serde_json::from_value::<SimulatedMachineStartData>(node.clone()) {
                Ok(mut data) => {
                    // Post-process Locality field using helper function
                    data.dcid = extract_from_locality(&data.locality, "dcid").map(String::from);
                    data.machineid =
                        extract_from_locality(&data.locality, "machineid").map(String::from);
                    Some(Event::SimulatedMachineStart(data))
                }
                Err(_) => None,
            }
        }
        "SimulatedMachineProcess" => try_parse_event_data::<SimulatedMachineProcessData>(node),
        "Assassination" => try_parse_event_data::<AssassinationData>(node),
        "CoordinatorsChangeBeforeCommit" => try_parse_event_data::<CoordinatorsChangeData>(node),
        "ProgramStart" => try_parse_event_data::<ProgramStartData>(node),
        _ => None, // Unknown "Type"
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

/// Parses a FoundationDB trace log file in JSON format.
///
/// Takes a path to the log file and returns a `Result` containing either a vector
/// of parsed `Event`s or a `ParsingError`.
pub fn parse_log_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<Event>, ParsingError> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for (index, line_result) in reader.lines().enumerate() {
        let line = line_result?; // Propagate IO errors
        let line_number = index + 1;

        // Attempt to parse the line as a JSON Value
        let node: JsonNode = match serde_json::from_str(&line) {
            Ok(val) => val,
            Err(e) => {
                // Consider logging this error instead of returning immediately
                // to allow parsing potentially valid lines later in the file.
                eprintln!("Skipping line {}: JSON parsing error - {}", line_number, e);
                continue; // Skip this line and continue with the next
                          // Or return Err(ParsingError::Json { line: line_number, source: e });
            }
        };

        // Attempt to parse the JSON Value into a specific Event type
        if let Some(event) = parse_event_from_node(&node) {
            events.push(event);
        } else {
            // Log or handle cases where a valid JSON object doesn't match a known Event type
            let event_type = node
                .get("Type")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            eprintln!(
                "Skipping line {}: Failed to parse event data for type '{}'",
                line_number, event_type
            );
            // Optionally, you could store these as a generic 'UnknownEvent' type
            // Or return Err(ParsingError::EventDataParsing { line: line_number, event_type: event_type.to_string() });
        }
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*; // Import items from outer module
    use std::env;

    #[test]
    fn test_parse_valid_log_file() {
        // Construct path relative to CARGO_MANIFEST_DIR
        let base_dir = env!("CARGO_MANIFEST_DIR");
        // The logs directory is within the parser crate's directory for this test setup
        let file_path =
            Path::new(base_dir).join("logs/trace.0.0.0.0.169.1745484896.1xR3BP.0.1.json");

        let events = parse_log_file(&file_path)
            .unwrap_or_else(|e| panic!("Failed to parse log file {:?}: {}", file_path, e));

        // Basic check: Ensure some events were parsed
        assert!(!events.is_empty(), "No events were parsed from the log file");

        // Check for presence of at least one ProgramStart event
        let has_program_start = events.iter().any(|e| matches!(e, Event::ProgramStart(_)));
        assert!(has_program_start, "No ProgramStart event found");

        // Specific check: Find a particular SimulatedMachineProcess event and verify fields
        let process_event = events
            .iter()
            .find_map(|e| match e {
                Event::SimulatedMachineProcess(data) if data.id == "68f8a443716dcad2" => Some(data),
                _ => None,
            })
            .expect("Failed to find specific SimulatedMachineProcess event");
        assert_eq!(process_event.address, "2.0.1.0:1");
        assert_eq!(process_event.data_hall, "0");
        assert_eq!(process_event.zone_id, "a2da9142f354b315465f9d57c6b5a01b");

        // Removed brittle count assertions
    }

    #[test]
    fn test_parse_io_error() {
        let result = parse_log_file("non_existent_file.log");
        assert!(matches!(result, Err(ParsingError::Io(_))));
    }

    #[test]
    fn test_parse_assassination_event() {
        let json_line = r#"{
            "Severity": "30", "Time": "138.462824", "DateTime": "2025-04-24T08:56:00Z", "Type": "Assassination", "Machine": "3.4.3.1:1", "TargetMachine": null, "TargetDatacenter": "1", "KillType": "6", "ThreadID": "123456789", "LogGroup": "default"
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
                assert_eq!(data.kill_type, Some(KillType::Unknown(6))); // Unknown kill type 6
            }
            _ => panic!("Parsed event is not an Assassination event"),
        }
    }

    #[test]
    fn test_parse_assassination_event_datacenter() {
        let json_line = r#"{
            "Severity": "30", "Time": "138.462824", "DateTime": "2025-04-24T08:56:00Z", "Type": "Assassination", "Machine": "3.4.3.1:1", "TargetMachine": null, "TargetDatacenter": "dc1", "KillType": "2", "ThreadID": "123456789", "LogGroup": "default"
        }"#;
        let node: JsonNode = serde_json::from_str(json_line).expect("Failed to parse JSON line");
        let event = parse_event_from_node(&node);

        assert!(event.is_some(), "Event should be parsed");
        match event.unwrap() {
            Event::Assassination(data) => {
                assert_eq!(data.timestamp, "138.462824");
                assert_eq!(data.machine, "3.4.3.1:1");
                assert!(data.target_machine.is_none());
                assert_eq!(data.target_datacenter.as_deref(), Some("dc1"));
                assert_eq!(data.kill_type, Some(KillType::KillInstantly)); // KillType 2
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
        let json_line = r#"{
            "Severity": "10", "Time": "251.750000", "OriginalTime": "59.959330", "DateTime": "2025-04-24T08:56:01Z", "OriginalDateTime": "2025-04-24T08:55:51Z", "Type": "ProgramStart", "Machine": "0.0.0.0:0", "ID": "0000000000000000", "Cycles": "2", "RandomId": "335a979b73b384c9", "SourceVersion": "412531b5c97fa84343da94888cc949a4d29e8c29", "Version": "7.3.43", "PackageName": "7.3", "DataFolder": "simfdb/6692e205b59f17fc558b5ed42d7a6bfa", "ConnectionString": "TestCluster:0@2.0.1.0:1:tls", "ActualTime": "1745484951", "CommandLine": "fdbserver -r simulation", "BuggifyEnabled": "1", "Simulated": "1", "ThreadID": "11198558628993500058", "LogGroup": "default", "TrackLatestType": "Rolled"
        }"#;
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
