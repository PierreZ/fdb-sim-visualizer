use serde::Serialize;

use crate::parser::{
    AssassinationData, ClogInterfaceData, CloggingPairData, CoordinatorsChangeData, Event,
};
use std::collections::HashMap;
use std::fmt;

// --- Report Structures ---

/// Contains details about a simulated machine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize)]
pub struct MachineInfo {
    pub id: String,
    pub ip_address: String,
    pub data_hall: Option<String>,
    pub dcid: Option<String>,
}

/// Holds summary statistics for CloggingPair events.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CloggingPairSummary {
    pub count: usize,
    pub min_seconds: f64,
    pub mean_seconds: f64,
    pub max_seconds: f64,
}

/// Holds summary statistics for ClogInterface events.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClogInterfaceSummary {
    pub count: usize,
    pub min_seconds: f64,
    pub mean_seconds: f64,
    pub max_seconds: f64,
}

/// Holds summary information extracted from a simulation log.
#[derive(Debug, Serialize)]
pub struct SimulationReport {
    /// The random seed used for the simulation run.
    pub seed: Option<String>,
    /// The total elapsed time reported by the simulation.
    pub elapsed_time: Option<String>,
    /// List of CloggingPair events, sorted by timestamp.
    pub clogging_pairs: Vec<CloggingPairData>,
    /// Summary statistics for CloggingPair events.
    pub clogging_pair_summary: Option<CloggingPairSummary>,
    /// List of ClogInterface events, sorted by timestamp.
    pub clog_interfaces: Vec<ClogInterfaceData>,
    /// Summary statistics for ClogInterface events, grouped by queue name.
    pub clog_interface_summary: HashMap<String, ClogInterfaceSummary>,
    /// List of Assassination events, sorted by timestamp.
    pub assassinations: Vec<AssassinationData>,
    /// Count of assassinations grouped by KillType.
    pub assassination_summary: HashMap<String, usize>,
    /// List of CoordinatorsChange events, sorted by timestamp.
    pub coordinators_changes: Vec<CoordinatorsChangeData>,
    /// Total count of coordinator changes.
    pub coordinators_change_count: usize,
    /// Details of machines involved in the simulation.
    pub machine_details: HashMap<String, MachineInfo>,
    /// Details of processes involved in the simulation.
    pub process_details: HashMap<String, ProcessInfo>,
}

impl fmt::Display for SimulationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "FoundationDB Simulation Report")?;
        writeln!(f, "==============================")?;

        if let Some(seed) = &self.seed {
            writeln!(f, "Seed: {}", seed)?;
        }
        if let Some(elapsed) = &self.elapsed_time {
            writeln!(f, "Elapsed Time: {} seconds", elapsed)?;
        }
        writeln!(f, "")?;

        writeln!(f, "--- Summaries ---")?;
        if let Some(summary) = &self.clogging_pair_summary {
            writeln!(f, "  Clogging Pairs:")?;
            writeln!(f, "    Count: {}", summary.count)?;
            writeln!(
                f,
                "    Duration (sec): Min={:.6}, Mean={:.6}, Max={:.6}",
                summary.min_seconds, summary.mean_seconds, summary.max_seconds
            )?;
        }

        writeln!(f, "  Clogged Interfaces (by Queue):")?;
        // Sort keys for consistent output
        let mut sorted_queues: Vec<_> = self.clog_interface_summary.keys().collect();
        sorted_queues.sort();
        for queue_name in sorted_queues {
            if let Some(summary) = self.clog_interface_summary.get(queue_name) {
                writeln!(f, "    Queue '{}':", queue_name)?;
                writeln!(f, "      Count: {}", summary.count)?;
                writeln!(
                    f,
                    "      Delay (sec): Min={:.6}, Mean={:.6}, Max={:.6}",
                    summary.min_seconds, summary.mean_seconds, summary.max_seconds
                )?;
            }
        }

        writeln!(f, "  Assassinations (by KillType):")?;
        if self.assassination_summary.is_empty() {
            writeln!(f, "    None")?;
        } else {
            // Sort keys for consistent output
            let mut sorted_kill_types: Vec<_> = self.assassination_summary.keys().collect();
            sorted_kill_types.sort();
            for kill_type in sorted_kill_types {
                if let Some(count) = self.assassination_summary.get(kill_type) {
                    writeln!(f, "    {}: {}", kill_type, count)?;
                }
            }
        }

        writeln!(
            f,
            "  Coordinator Changes: {}",
            self.coordinators_change_count
        )?;
        writeln!(f, "")?;

        // Initially, don't print the long vectors or detailed maps
        // Add sections here later if needed, possibly behind a flag.
        writeln!(f, "--- Details ---")?;
        writeln!(f, "  Machines Found: {}", self.machine_details.len())?;
        writeln!(f, "  Processes Found: {}", self.process_details.len())?;
        writeln!(
            f,
            "(Raw event lists and detailed machine info omitted for brevity)"
        )?;

        Ok(())
    }
}

/// Creates a `SimulationReport` by processing a slice of `Event`s.
///
/// Extracts the seed, a list of unique machine identifiers (from ProgramStart events),
/// the last reported elapsed time, and groups specific events into time-ordered vectors.
pub fn create_simulation_report(events: &[Event]) -> SimulationReport {
    let mut seed = None;
    // Removed machine_set
    let mut elapsed_time = None;

    // Use HashMaps to collect unique machine details
    let mut machine_details: HashMap<String, MachineInfo> = HashMap::new();
    let mut process_details: HashMap<String, ProcessInfo> = HashMap::new();

    // Initialize vectors for other event types
    let mut clogging_pairs = Vec::new();
    let mut clog_interfaces = Vec::new();
    let mut assassinations = Vec::new();
    let mut coordinators_changes = Vec::new();

    for event in events {
        match event {
            Event::ProgramStart(data) => {
                // Only take the seed from the first ProgramStart event
                if seed.is_none() && data.random_seed.is_some() {
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
                let machine_info = MachineInfo {
                    id: data.id.clone(),
                    ip_address: data.machine_ips.clone(), // Assuming single IP for now
                    data_hall: Some(data.data_hall.clone()),
                    dcid: data.dcid.clone(),
                };
                machine_details.insert(data.id.clone(), machine_info);
            }
            Event::SimulatedMachineProcess(data) => {
                let process_info = ProcessInfo {
                    process_address: data.address.clone(), // Use data.address
                    machine_id: data.id.clone(),           // Use data.id
                };
                process_details.insert(data.address.clone(), process_info); // Use data.address as key
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
        assassination_summary, // Include assassination summary
        coordinators_changes,
        coordinators_change_count, // Include coordinator change count
        machine_details,           // Use map directly
        process_details,           // Include process details
    }
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*; // Import items from outer module (report)
    use crate::parser::parse_log_file; // Import parse_log_file
    use std::path::Path; // Need Path for parse_log_file

    #[test]
    fn test_create_report_from_log() {
        // Define the path relative to the crate root (parser directory)
        let log_path_str = "logs/combined_trace.0.0.0.0.24.1745498878.p7Loj0.json";
        // Use Path relative to crate root (which is 'parser' when running tests)
        let log_path = Path::new(log_path_str);

        // Use parse_log_file to load events
        let events = parse_log_file(log_path).expect(&format!(
            "Failed to parse log file '{}' using parse_log_file",
            log_path.display()
        ));

        // Create the report
        let report = create_simulation_report(&events);

        // Assertions for basic report info
        assert_eq!(report.seed, Some("292006968".to_string()));
        assert_eq!(report.elapsed_time, Some("351.752".to_string()));

        // Assertions for Clogging Pairs
        assert_eq!(report.clogging_pairs.len(), 396);
        assert!(report.clogging_pair_summary.is_some());
        let clog_pair_summary = report.clogging_pair_summary.as_ref().unwrap();
        assert_eq!(clog_pair_summary.count, 396);
        assert!((clog_pair_summary.min_seconds - 0.000720).abs() < 1e-6);
        assert!((clog_pair_summary.mean_seconds - 0.633275).abs() < 1e-6);
        assert!((clog_pair_summary.max_seconds - 6.198240).abs() < 1e-6);

        // Assertions for Clogged Interfaces
        // Removed assertion for raw vector length: assert_eq!(report.clog_interfaces.len(), 189);
        assert_eq!(report.clog_interface_summary.len(), 3); // 'All', 'Receive', 'Send'

        // Check 'All' queue summary
        assert!(report.clog_interface_summary.contains_key("All"));
        let all_summary = report.clog_interface_summary.get("All").unwrap();
        assert_eq!(all_summary.count, 189);
        assert!((all_summary.min_seconds - 0.000051).abs() < 1e-6);
        assert!((all_summary.mean_seconds - 0.263599).abs() < 1e-6);
        assert!((all_summary.max_seconds - 4.536360).abs() < 1e-6);

        // Check 'Receive' queue summary
        assert!(report.clog_interface_summary.contains_key("Receive"));
        let receive_summary = report.clog_interface_summary.get("Receive").unwrap();
        assert_eq!(receive_summary.count, 135);
        assert!((receive_summary.min_seconds - 0.000049).abs() < 1e-6);
        assert!((receive_summary.mean_seconds - 0.325336).abs() < 1e-6);
        assert!((receive_summary.max_seconds - 4.316820).abs() < 1e-6);

        // Check 'Send' queue summary
        assert!(report.clog_interface_summary.contains_key("Send"));
        let send_summary = report.clog_interface_summary.get("Send").unwrap();
        assert_eq!(send_summary.count, 157);
        assert!((send_summary.min_seconds - 0.000158).abs() < 1e-6);
        assert!((send_summary.mean_seconds - 0.361933).abs() < 1e-6);
        assert!((send_summary.max_seconds - 4.221570).abs() < 1e-6);

        // Assertions for Assassinations
        assert_eq!(report.assassinations.len(), 10);
        assert_eq!(report.assassination_summary.len(), 1);
        assert!(report.assassination_summary.contains_key("Unknown"));
        assert_eq!(report.assassination_summary.get("Unknown"), Some(&10));

        // Assertions for Coordinator Changes
        assert_eq!(report.coordinators_changes.len(), 1);
        assert_eq!(report.coordinators_change_count, 1);

        // Assertions for Machine Details
        assert_eq!(report.machine_details.len(), 23);
        // Check a specific process (using address)
        assert!(report.process_details.contains_key("2.0.1.0:1"));
        let process_info = report.process_details.get("2.0.1.0:1").unwrap();
        assert_eq!(process_info.process_address, "2.0.1.0:1");
        assert_eq!(process_info.machine_id, "e2962137f7c53240");

        // Assertions for Assassinations
        assert_eq!(report.assassinations.len(), 10);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProcessInfo {
    pub process_address: String,
    pub machine_id: String,
}
