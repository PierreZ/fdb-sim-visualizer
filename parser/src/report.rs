use crate::parser::Event;
use crate::parser::{
    AssassinationData, ClogInterfaceData, CloggingPairData, CoordinatorsChangeData,
};
use std::collections::HashMap;

/// Contains details about a simulated machine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)] // Keep Ord/PartialOrd for potential future use if needed
pub struct MachineInfo {
    pub id: String,
    pub data_hall: String,
    pub dcid: Option<String>,
}

/// Contains details about a simulated process.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)] // Keep Ord/PartialOrd
pub struct ProcessInfo {
    pub id: String,
    pub address: String,
    pub data_hall: String,
    pub zone_id: String,
}

/// Holds summary information extracted from a simulation log.
#[derive(Debug)]
pub struct SimulationReport {
    /// The random seed used for the simulation run.
    pub seed: Option<String>,
    // Removed machines: Vec<String>
    /// The final elapsed simulation time reported.
    pub elapsed_time: Option<String>,
    // Separate, time-ordered vectors for specific event types
    pub clogging_pairs: Vec<CloggingPairData>,
    pub clog_interfaces: Vec<ClogInterfaceData>,
    pub assassinations: Vec<AssassinationData>,
    pub coordinators_changes: Vec<CoordinatorsChangeData>,
    // Processed details for machines and processes (using Maps)
    pub machine_details: HashMap<String, MachineInfo>, // Changed to HashMap (Key: Machine ID)
    pub process_details: HashMap<String, ProcessInfo>, // Changed to HashMap (Key: Process Address)
}

/// Creates a `SimulationReport` by processing a slice of `Event`s.
///
/// Extracts the seed, a list of unique machine identifiers (from ProgramStart events),
/// the last reported elapsed time, and groups specific events into time-ordered vectors.
pub fn create_simulation_report(events: &[Event]) -> SimulationReport {
    let mut seed = None;
    // Removed machine_set
    let mut elapsed_time = None;

    // Use HashMaps to collect unique machine/process details
    let mut machine_details_map: HashMap<String, MachineInfo> = HashMap::new();
    let mut process_details_map: HashMap<String, ProcessInfo> = HashMap::new();

    // Initialize vectors for other event types
    let mut clogging_pairs = Vec::new();
    let mut clog_interfaces = Vec::new();
    let mut assassinations = Vec::new();
    let mut coordinators_changes = Vec::new();

    for event in events {
        match event {
            Event::ProgramStart(data) => {
                if data.random_seed.is_some() {
                    seed = data.random_seed.clone();
                }
                // No longer collecting machine names here
            }
            Event::ElapsedTime(data) => {
                elapsed_time = Some(data.sim_time.clone());
            }
            Event::CloggingPair(data) => clogging_pairs.push(data.clone()),
            Event::ClogInterface(data) => clog_interfaces.push(data.clone()),
            Event::SimulatedMachineStart(data) => {
                // Use machine ID as key
                machine_details_map
                    .entry(data.id.clone())
                    .or_insert_with(|| MachineInfo {
                        id: data.id.clone(),
                        data_hall: data.data_hall.clone(),
                        dcid: data.dcid.clone(),
                    });
            }
            Event::SimulatedMachineProcess(data) => {
                // Use process address as key
                process_details_map
                    .entry(data.address.clone())
                    .or_insert_with(|| ProcessInfo {
                        id: data.id.clone(),
                        address: data.address.clone(),
                        data_hall: data.data_hall.clone(),
                        zone_id: data.zone_id.clone(),
                    });
            }
            Event::Assassination(data) => assassinations.push(data.clone()),
            Event::CoordinatorsChange(data) => coordinators_changes.push(data.clone()),
        }
    }

    // --- Sorting Logic for Vecs ---
    let parse_ts = |ts_str: &str| ts_str.parse::<f64>().unwrap_or(0.0);

    clogging_pairs.sort_by(|a, b| {
        parse_ts(&a.timestamp)
            .partial_cmp(&parse_ts(&b.timestamp))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    clog_interfaces.sort_by(|a, b| {
        parse_ts(&a.timestamp)
            .partial_cmp(&parse_ts(&b.timestamp))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    assassinations.sort_by(|a, b| {
        parse_ts(&a.timestamp)
            .partial_cmp(&parse_ts(&b.timestamp))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    coordinators_changes.sort_by(|a, b| {
        parse_ts(&a.timestamp)
            .partial_cmp(&parse_ts(&b.timestamp))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // No need to sort HashMaps or convert them

    // Removed machine list conversion

    SimulationReport {
        seed,
        // machines field removed
        elapsed_time,
        clogging_pairs,
        clog_interfaces,
        assassinations,
        coordinators_changes,
        machine_details: machine_details_map, // Use map directly
        process_details: process_details_map, // Use map directly
    }
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*; // Import items from outer module (report)
    use crate::parser::parse_log_file; // Import the parser function
    use std::path::Path;

    #[test]
    fn test_create_report_from_log() {
        // Define the path relative to the crate root (parser directory)
        let log_path_str = "logs/trace.0.0.0.0.169.1745484896.1xR3BP.0.1.json";
        let log_path = Path::new(log_path_str);

        // Check if the log file exists before parsing
        if !log_path.exists() {
            panic!("Test log file not found at '{}'. Make sure the path is relative to the 'parser' crate root.", log_path_str);
        }

        // Parse the log file
        let events = parse_log_file(log_path).expect("Failed to parse log file");
        assert!(!events.is_empty(), "Parser returned no events.");

        // Create the report
        let report = create_simulation_report(&events);
        dbg!(&report); // Debug print the created report

        // Assertions
        assert_eq!(
            report.seed,
            Some("2660455843".to_string()),
            "Seed does not match expected value from log."
        );
        assert!(
            report.elapsed_time.is_some(),
            "Elapsed time should be present."
        );

        // Check machine details (basic checks)
        assert!(
            !report.machine_details.is_empty(),
            "Machine details map should not be empty."
        );
        // Example: Check if a specific machine ID expected from this log exists
        // assert!(report.machine_details.contains_key("machine_id_example"), "Expected machine ID not found.");
        // Add more specific checks based on log content if needed

        // Check process details (basic checks)
        assert!(
            !report.process_details.is_empty(),
            "Process details map should not be empty."
        );
        // Example: Check if a specific process address expected from this log exists
        // assert!(report.process_details.contains_key("1.2.3.4:4500"), "Expected process address not found.");
        // Add more specific checks based on log content if needed

        // Check other event vectors (optional, check if they are populated as expected)
        // assert!(!report.clogging_pairs.is_empty(), "Clogging pairs should exist in this log.");
        // assert!(!report.assassinations.is_empty(), "Assassinations should exist in this log.");
    }
}
