use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonNode;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

/// Represents different types of log events.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Event {
    /// Represents a CloggingPair event.
    CloggingPair(CloggingPairData),
    /// Represents a ClogInterface event.
    ClogInterface(ClogInterfaceData),
    /// Represents an ElapsedTime event.
    ElapsedTime(ElapsedTimeData),
    /// Represents a SimulatedMachineStart event.
    SimulatedMachineStart(SimulatedMachineStartData),
    /// Represents a CoordinatorsChange event.
    CoordinatorsChange(CoordinatorsChangeData),
    /// Represents a ProgramStart event.
    ProgramStart(ProgramStartData),
    /// Represents a DiskSwap event.
    DiskSwap(DiskSwapData),
    /// Represents a SetDiskFailure event.
    SetDiskFailure(SetDiskFailureData),
    /// Represents a CorruptedBlock event.
    CorruptedBlock(CorruptedBlockData),
    /// Represents a KillMachineProcess event.
    KillMachineProcess(KillMachineProcessData),
    // Add other specific event variants here
}

/// Data specific to a CloggingPair event.
#[derive(Debug, Deserialize, PartialEq, Clone, Serialize)]
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
#[derive(Debug, Deserialize, PartialEq, Clone, Serialize)]
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
#[derive(Debug, Deserialize, PartialEq, Clone, Serialize)]
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
#[derive(Debug, Deserialize, PartialEq, Clone, Serialize)]
pub struct SimulatedMachineStartData {
    #[serde(rename = "Time")]
    pub timestamp: String,
    #[serde(rename = "ProcessClass")]
    pub process_class: String,
    #[serde(rename = "Locality")]
    pub locality: String, // Contains dcid, machineid, etc.
    // fields populated from Locality
    pub zone_id: Option<String>,
    pub process_id: Option<String>,
    pub machine_id: Option<String>,
    pub dc_id: Option<String>,
    pub data_hall: Option<String>,
}

impl SimulatedMachineStartData {
    fn populate_from_locality(&mut self) {
        let locality_string = self.locality.split(" ");
        for part in locality_string {
            // split again
            let (key, value) = part.split_once("=").unwrap();
            let parsed_value = if value == "[unset]" {
                None
            } else {
                Some(value.to_string())
            };
            match key {
                "zoneid" => self.zone_id = parsed_value,
                "processid" => self.process_id = parsed_value,
                "machineid" => self.machine_id = parsed_value,
                "dcid" => self.dc_id = parsed_value,
                "data_hall" => self.data_hall = parsed_value,
                _ => {}
            }
        }
    }
}

impl Into<Event> for SimulatedMachineStartData {
    fn into(self) -> Event {
        Event::SimulatedMachineStart(self)
    }
}

/// Data specific to a CoordinatorsChange event.
#[derive(Debug, Deserialize, PartialEq, Clone, Serialize)]
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
#[derive(Debug, Deserialize, PartialEq, Clone, Serialize)]
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

/// Data specific to a DiskSwap (SimulatedMachineFolderSwap) event.
#[derive(Debug, Deserialize, PartialEq, Clone, Serialize)]
pub struct DiskSwapData {
    #[serde(rename = "Time")]
    pub timestamp: String,
    #[serde(rename = "MachineIPs")]
    pub machine_ips: String, // Assuming this is a single string like "[ip1,ip2,...]"
                             // Consider parsing this further if needed
}

impl Into<Event> for DiskSwapData {
    fn into(self) -> Event {
        Event::DiskSwap(self)
    }
}

/// Data specific to a SetDiskFailure event.
#[derive(Debug, Deserialize, PartialEq, Clone, Serialize)]
pub struct SetDiskFailureData {
    #[serde(rename = "Time")]
    pub timestamp: String,
    #[serde(rename = "Machine")]
    pub machine: String,
    #[serde(rename = "StallInterval")]
    pub stall_interval: String,
    #[serde(rename = "StallPeriod")]
    pub stall_period: String,
    #[serde(rename = "StallUntil")]
    pub stall_until: String,
    #[serde(rename = "ThrottlePeriod")]
    pub throttle_period: String,
    #[serde(rename = "ThrottleUntil")]
    pub throttle_until: String,
    // Other fields ignored for now: Severity, DateTime, ID, Now, ThreadID, LogGroup, Roles
}

impl Into<Event> for SetDiskFailureData {
    fn into(self) -> Event {
        Event::SetDiskFailure(self)
    }
}

/// Data specific to a CorruptedBlock event.
#[derive(Debug, Deserialize, PartialEq, Clone, Serialize)]
pub struct CorruptedBlockData {
    #[serde(rename = "Severity")]
    pub severity: String,
    #[serde(rename = "Time")]
    pub time: String,
    #[serde(rename = "DateTime")]
    pub date_time: String,
    #[serde(rename = "Type")]
    pub event_type: String, // Should always be "CorruptedBlock"
    #[serde(rename = "Machine")]
    pub machine: String,
    #[serde(rename = "Filename")]
    pub filename: String,
    #[serde(rename = "Block")]
    pub block: String, // Keep as string, might not always be numeric?
    #[serde(rename = "ID")] // Optional, seems to be 000... in example
    pub id: Option<String>,
    #[serde(rename = "Roles")] // Optional
    pub roles: Option<String>,
}

impl Into<Event> for CorruptedBlockData {
    fn into(self) -> Event {
        Event::CorruptedBlock(self)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KillType {
    KillInstantly,          // 0
    InjectFaults,           // 1
    FailDisk,               // 2
    RebootAndDelete,        // 3
    RebootProcessAndDelete, // 4
    RebootProcessAndSwitch, // 5
    Reboot,                 // 6
    RebootProcess,          // 7
    None,                   // 8
    Unknown,                // Added for parsing errors
}

impl FromStr for KillType {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = s.parse::<u8>()?;
        match val {
            0 => Ok(KillType::KillInstantly),
            1 => Ok(KillType::InjectFaults),
            2 => Ok(KillType::FailDisk),
            3 => Ok(KillType::RebootAndDelete),
            4 => Ok(KillType::RebootProcessAndDelete),
            5 => Ok(KillType::RebootProcessAndSwitch),
            6 => Ok(KillType::Reboot),
            7 => Ok(KillType::RebootProcess),
            _ => Ok(KillType::Unknown),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone, Serialize)]
pub struct KillMachineProcessData {
    #[serde(rename = "Time")]
    pub timestamp: String,
    #[serde(rename = "KillType")]
    pub raw_kill_type: String,
    #[serde(rename = "Process")]
    pub process: String,
    #[serde(rename = "StartingClass")]
    pub starting_class: String,
    #[serde(rename = "Failed")]
    pub failed: String,
    #[serde(rename = "Excluded")]
    pub excluded: String,
    #[serde(rename = "Cleared")]
    pub cleared: String,
    #[serde(rename = "Rebooting")]
    pub rebooting: String,
}

impl Into<Event> for KillMachineProcessData {
    fn into(self) -> Event {
        Event::KillMachineProcess(self)
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
            Event::CoordinatorsChange(data) => data.timestamp.parse().unwrap_or(0.0),
            Event::ProgramStart(data) => data.timestamp.parse().unwrap_or(0.0),
            Event::DiskSwap(data) => data.timestamp.parse().unwrap_or(0.0),
            Event::SetDiskFailure(data) => data.timestamp.parse().unwrap_or(0.0),
            Event::CorruptedBlock(data) => data.time.parse().unwrap_or(0.0),
            Event::KillMachineProcess(data) => data.timestamp.parse().unwrap_or(0.0),
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
    serde_json::from_value::<T>(node.clone())
        .ok()
        .map(Into::into)
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
                    data.populate_from_locality();
                    Some(Event::SimulatedMachineStart(data))
                }
                Err(_) => None,
            }
        }
        "CoordinatorsChangeBeforeCommit" => try_parse_event_data::<CoordinatorsChangeData>(node),
        "ProgramStart" => try_parse_event_data::<ProgramStartData>(node).map(|e| e.into()),
        "SimulatedMachineFolderSwap" => {
            try_parse_event_data::<DiskSwapData>(node).map(|e| e.into())
        } // Use DiskSwapData struct
        "SetDiskFailure" => try_parse_event_data::<SetDiskFailureData>(node).map(|e| e.into()),
        "CorruptedBlock" => try_parse_event_data::<CorruptedBlockData>(node).map(|e| e.into()),
        "KillMachineProcess" => {
            try_parse_event_data::<KillMachineProcessData>(node).map(|e| e.into())
        }
        _ => None, // Unknown event type
    }
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
            // let event_type = node
            //     .get("Type")
            //     .and_then(|v| v.as_str())
            //     .unwrap_or("Unknown");
            // eprintln!(
            //     "Skipping line {}: Failed to parse event data for type '{}'",
            //     line_number, event_type
            // );
            // Optionally, you could store these as a generic 'UnknownEvent' type
            // Or return Err(ParsingError::EventDataParsing { line: line_number, event_type: event_type.to_string() });
        }
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*; // Import items from outer module
    use serde_json::json;
    use std::path::Path;

    #[test]
    fn test_parse_valid_log_file() {
        // Define the path relative to the crate root (parser directory)
        let log_path_str = "logs/combined_trace.0.0.0.0.24.1745498878.p7Loj0.json";
        let log_path = Path::new(log_path_str);

        // Check if the log file exists before parsing
        assert!(log_path.exists(), "Log file not found: {}", log_path_str);

        // Parse the log file
        let events = parse_log_file(log_path)
            .expect(&format!("Failed to parse log file \"{}\"", log_path_str));
        assert!(!events.is_empty(), "Parser returned no events.");
        // Add more specific assertions based on expected events if needed
    }

    #[test]
    fn test_parse_io_error() {
        let result = parse_log_file("non_existent_file.log");
        assert!(matches!(result, Err(ParsingError::Io(_))));
    }

    #[test]
    fn test_parse_program_start_event_with_seed() {
        let json_line = json!({
          "Severity": "10", "Time": "0.000000", "DateTime": "2025-04-24T08:55:36Z", "Type": "ProgramStart", "Machine": "0.0.0.0:0", "ID": "0000000000000000", "RandomSeed": "2837976339", "SourceVersion": "412531b5c97fa84343da94888cc949a4d29e8c29", "Version": "7.3.43", "PackageName": "7.3", "FileSystem": "", "DataFolder": "", "WorkingDirectory": "/root", "ClusterFile": "", "ConnectionString": "", "ActualTime": "1745484936", "EnvironmentKnobOptions": "none", "CommandLine": "fdbserver -r simulation -f /root/logical_db.toml -b on --trace-format json -L ./logs", "BuggifyEnabled": "1", "FaultInjectionEnabled": "1", "MemoryLimit": "8589934592", "VirtualMemoryLimit": "0", "ProtocolVersion": "0x0FDB00B073000000", "ThreadID": "10058538621798076542", "LogGroup": "default", "TrackLatestType": "Original"
        });
        let node: JsonNode =
            serde_json::from_str(&json_line.to_string()).expect("Failed to parse JSON line");
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
        let json_line = json!({
          "Severity": "10", "Time": "251.750000", "OriginalTime": "59.959330", "DateTime": "2025-04-24T08:56:01Z", "OriginalDateTime": "2025-04-24T08:55:51Z", "Type": "ProgramStart", "Machine": "0.0.0.0:0", "ID": "0000000000000000", "Cycles": "2", "RandomId": "335a979b73b384c9", "SourceVersion": "412531b5c97fa84343da94888cc949a4d29e8c29", "Version": "7.3.43", "PackageName": "7.3", "DataFolder": "simfdb/6692e205b59f17fc558b5ed42d7a6bfa", "ConnectionString": "TestCluster:0@2.0.1.0:1:tls", "ActualTime": "1745484951", "CommandLine": "fdbserver -r simulation", "BuggifyEnabled": "1", "Simulated": "1", "ThreadID": "11198558628993500058", "LogGroup": "default", "TrackLatestType": "Rolled"
        });
        let node: JsonNode =
            serde_json::from_str(&json_line.to_string()).expect("Failed to parse JSON line");
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

    #[test]
    fn test_parse_set_disk_failure_event() {
        let json_data = json!({
          "Severity": "10",
          "Time": "146.900748",
          "DateTime": "2025-04-25T09:26:05Z",
          "Type": "SetDiskFailure",
          "Machine": "2.1.1.0:1",
          "ID": "0000000000000000",
          "Now": "146.901",
          "StallInterval": "5",
          "StallPeriod": "5",
          "StallUntil": "151.901",
          "ThrottlePeriod": "30",
          "ThrottleUntil": "176.901",
          "ThreadID": "12256871313368394809",
          "LogGroup": "default",
          "Roles": "CD,LR,SS,TL"
        });

        let expected_event = Event::SetDiskFailure(SetDiskFailureData {
            timestamp: "146.900748".to_string(),
            machine: "2.1.1.0:1".to_string(),
            stall_interval: "5".to_string(),
            stall_period: "5".to_string(),
            stall_until: "151.901".to_string(),
            throttle_period: "30".to_string(),
            throttle_until: "176.901".to_string(),
        });

        let parsed_event = parse_event_from_node(&json_data);
        assert_eq!(parsed_event, Some(expected_event));
    }

    #[test]
    fn test_parse_corrupted_block_event() {
        let json_line = json!({
          "Severity": "10", "Time": "93.070647", "DateTime": "2025-04-25T09:40:11Z", "Type": "CorruptedBlock", "Machine": "2.0.1.3:1", "ID": "0000000000000000", "Filename": "/path/to/storage.sqlite", "Block": "20", "ThreadID": "123", "LogGroup": "default", "Roles": "BK,CP,SS,TL"
        });
        let node: JsonNode =
            serde_json::from_str(&json_line.to_string()).expect("Failed to parse JSON line");
        let event = parse_event_from_node(&node);

        assert!(event.is_some(), "Event should be parsed");
        match event.unwrap() {
            Event::CorruptedBlock(data) => {
                assert_eq!(data.time, "93.070647");
                assert_eq!(data.machine, "2.0.1.3:1");
                assert_eq!(data.filename, "/path/to/storage.sqlite");
                assert_eq!(data.block, "20");
                assert_eq!(data.roles, Some("BK,CP,SS,TL".to_string()));
            }
            _ => panic!("Parsed event is not a CorruptedBlock event"),
        }
    }

    #[test]
    fn test_parse_unknown_event() {
        let json_line = json!({
          "Severity": "10", "Time": "93.070647", "DateTime": "2025-04-25T09:40:11Z", "Type": "UnknownEvent", "Machine": "2.0.1.3:1", "ID": "0000000000000000", "Filename": "/path/to/storage.sqlite", "Block": "20", "ThreadID": "123", "LogGroup": "default", "Roles": "BK,CP,SS,TL"
        });
        let node: JsonNode =
            serde_json::from_str(&json_line.to_string()).expect("Failed to parse JSON line");
        let event = parse_event_from_node(&node);

        assert!(event.is_none(), "Event should not be parsed");
    }

    #[test]
    fn test_parse_kill_machine_process_event() {
        let json_data = json!({
          "Severity": "10",
          "Time": "10.0",
          "DateTime": "2024-07-29T16:30:00Z",
          "Type": "KillMachineProcess",
          "Machine": "127.0.0.1:4000",
          "ID": "test_id",
          "LogGroup": "default",
          "Roles": "SS",
          "TrackLatest": "",
          "KillType": "6", // Represents Reboot
          "Process": "127.0.0.1:4001",
          "StartingClass": "Storage",
          "Failed": "false",
          "Excluded": "false",
          "Cleared": "false",
          "Rebooting": "true"
        });

        let event = parse_event_from_node(&json_data).unwrap();
        let expected_data = KillMachineProcessData {
            timestamp: "10.0".to_string(),
            raw_kill_type: "6".to_string(),
            process: "127.0.0.1:4001".to_string(),
            starting_class: "Storage".to_string(),
            failed: "false".to_string(),
            excluded: "false".to_string(),
            cleared: "false".to_string(),
            rebooting: "true".to_string(),
        };

        assert_eq!(event, Event::KillMachineProcess(expected_data));
    }

    #[test]
    fn test_kill_type_from_str() {
        assert_eq!(KillType::from_str("0").unwrap(), KillType::KillInstantly);
        assert_eq!(KillType::from_str("1").unwrap(), KillType::InjectFaults);
        assert_eq!(KillType::from_str("2").unwrap(), KillType::FailDisk);
        assert_eq!(KillType::from_str("3").unwrap(), KillType::RebootAndDelete);
        assert_eq!(
            KillType::from_str("4").unwrap(),
            KillType::RebootProcessAndDelete
        );
        assert_eq!(
            KillType::from_str("5").unwrap(),
            KillType::RebootProcessAndSwitch
        );
        assert_eq!(KillType::from_str("6").unwrap(), KillType::Reboot);
        assert_eq!(KillType::from_str("7").unwrap(), KillType::RebootProcess);
        assert_eq!(KillType::from_str("8").unwrap(), KillType::Unknown);
    }

    // ... other tests remain unchanged ...
}
