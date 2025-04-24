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

/// Holds summary statistics for CloggingPair events.
#[derive(Debug, Clone, PartialEq)]
pub struct CloggingPairSummary {
    pub count: usize,
    pub min_seconds: f64,
    pub mean_seconds: f64,
    pub max_seconds: f64,
}

/// Holds summary statistics for ClogInterface events.
#[derive(Debug, Clone, PartialEq)]
pub struct ClogInterfaceSummary {
    pub count: usize,
    pub min_seconds: f64,
    pub mean_seconds: f64,
    pub max_seconds: f64,
}

/// Holds summary information extracted from a simulation log.
#[derive(Debug)]
pub struct SimulationReport {
    /// The random seed used for the simulation run.
    pub seed: Option<String>,
    /// The final elapsed simulation time reported.
    pub elapsed_time: Option<String>,
    // Separate, time-ordered vectors for specific event types
    pub clogging_pairs: Vec<CloggingPairData>,
    pub clogging_pair_summary: Option<CloggingPairSummary>, // Renamed summary field
    pub clog_interfaces: Vec<ClogInterfaceData>,
    // Group interface summary by queue name
    pub clog_interface_summary: HashMap<String, ClogInterfaceSummary>,
    pub assassinations: Vec<AssassinationData>,
    pub coordinators_changes: Vec<CoordinatorsChangeData>,
    pub coordinators_change_count: usize,
    // Summary of assassinations by KillType
    pub assassination_summary: HashMap<String, usize>,
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

    // --- Calculate Clogging Summary ---
    let mut min_seconds = f64::MAX;
    let mut max_seconds = f64::MIN;
    let mut sum_seconds = 0.0;
    let mut count = 0;

    for pair in &clogging_pairs {
        // Attempt to parse the seconds string into f64
        if let Ok(seconds) = pair.seconds.parse::<f64>() {
            min_seconds = min_seconds.min(seconds);
            max_seconds = max_seconds.max(seconds);
            sum_seconds += seconds;
            count += 1;
        } else {
            // Optional: Log warning for parse failures if needed
            // eprintln!("Warning: Failed to parse clogging seconds '{}' for pair {:?}", pair.seconds, pair);
        }
    }

    let clogging_pair_summary = if count > 0 {
        Some(CloggingPairSummary {
            count,
            min_seconds,
            mean_seconds: sum_seconds / count as f64,
            max_seconds,
        })
    } else {
        // Provide a default summary if no valid clogging pairs were found
        Some(CloggingPairSummary {
            count: 0,
            min_seconds: 0.0,
            mean_seconds: 0.0,
            max_seconds: 0.0,
        })
    };

    // --- Calculate Clog Interface Summary (Grouped by Queue) ---
    let mut interface_stats: HashMap<String, (f64, f64, f64, usize)> = HashMap::new(); // (sum, min, max, count)

    for interface in &clog_interfaces {
        if let Ok(seconds) = interface.delay.parse::<f64>() {
            let queue_name = interface.queue.clone(); // Use queue name as key
            let entry = interface_stats
                .entry(queue_name)
                .or_insert((0.0, f64::MAX, f64::MIN, 0));
            entry.0 += seconds; // sum
            entry.1 = entry.1.min(seconds); // min
            entry.2 = entry.2.max(seconds); // max
            entry.3 += 1; // count
        } // Optional: Log warning for parse failures
    }

    let clog_interface_summary: HashMap<String, ClogInterfaceSummary> = interface_stats
        .into_iter()
        .map(|(queue, (sum, min_val, max_val, count))| {
            let summary = ClogInterfaceSummary {
                count,
                min_seconds: min_val,
                mean_seconds: if count > 0 { sum / count as f64 } else { 0.0 },
                max_seconds: max_val,
            };
            (queue, summary)
        })
        .collect();

    // --- Calculate Assassination Summary (Grouped by KillType) ---
    let mut assassination_summary: HashMap<String, usize> = HashMap::new();
    for assassination in &assassinations {
        // Convert KillType (or None) to a string key
        let key = match &assassination.kill_type {
            Some(kt) => format!("{:?}", kt), // Use Debug representation for now
            None => "Unknown".to_string(),
        };
        *assassination_summary.entry(key).or_insert(0) += 1;
    }

    // --- Calculate Coordinator Change Count ---
    let coordinators_change_count = coordinators_changes.len();

    // No need to sort HashMaps or convert them

    SimulationReport {
        seed,
        // machines field removed
        elapsed_time,
        clogging_pairs,
        clogging_pair_summary, // Include renamed summary in report
        clog_interfaces,
        clog_interface_summary, // Include new summary
        assassinations,
        coordinators_changes,
        coordinators_change_count, // Include coordinator change count
        assassination_summary,     // Include assassination summary
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
        // dbg!(&report); // Optionally uncomment to debug print the full report
        dbg!(report.clogging_pair_summary.as_ref()); // Debug print just the summary
        dbg!(&report.clog_interface_summary); // Debug print interface summary
        dbg!(&report.assassination_summary); // Debug print assassination summary
        dbg!(report.coordinators_change_count); // Debug print coordinator change count

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

        // Check clogging summary
        assert!(
            report.clogging_pair_summary.is_some(),
            "Clogging summary should be present."
        );
        // Check specific values for CloggingPairSummary (adjust precision as needed)
        if let Some(summary) = report.clogging_pair_summary.as_ref() {
            assert_eq!(summary.count, 308, "Incorrect clogging pair count");
            assert!(
                (summary.min_seconds - 0.000734521).abs() < 1e-9,
                "Incorrect min clogging pair seconds"
            );
            assert!(
                (summary.mean_seconds - 1.1832040077597417).abs() < 1e-9,
                "Incorrect mean clogging pair seconds"
            );
            assert!(
                (summary.max_seconds - 9.04286).abs() < 1e-9,
                "Incorrect max clogging pair seconds"
            );
        }

        // Check clog interface summary (now a map)
        assert!(
            !report.clog_interface_summary.is_empty(),
            "Clog interface summary map should not be empty."
        );
        // Optionally, assert presence and values for specific queues if known
        // e.g., assert!(report.clog_interface_summary.contains_key("SpecificQueueName"));

        // Check assassination summary
        dbg!(&report.assassination_summary);
        assert!(
            !report.assassination_summary.is_empty(),
            "Assassination summary map should not be empty."
        );
        assert_eq!(
            report.assassination_summary.len(),
            1,
            "Assassination summary map should have exactly one entry."
        );
        assert_eq!(
            report.assassination_summary.get("RebootProcess"),
            Some(&1),
            "Incorrect count for KillType RebootProcess"
        );

        // Check coordinator change count
        dbg!(report.coordinators_change_count);
        assert_eq!(
            report.coordinators_change_count, 1,
            "Incorrect coordinator change count."
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
