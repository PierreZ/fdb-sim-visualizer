use serde::{Deserialize, Serialize};

use crate::parser::{
    ClogInterfaceData, CloggingPairData, CoordinatorsChangeData,
    CorruptedBlockData, DiskSwapData, Event, SetDiskFailureData,
};
use std::collections::HashMap;
use std::fmt;

// --- Summary Structs (Restored) --- //

/// Holds summary statistics for CloggingPair events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloggingPairSummary {
    pub count: usize,
    pub min_seconds: f64,
    pub mean_seconds: f64,
    pub max_seconds: f64,
}

/// Holds summary statistics for ClogInterface events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClogInterfaceSummary {
    pub count: usize,
    pub min_seconds: f64,
    pub mean_seconds: f64,
    pub max_seconds: f64,
}

/// Holds details about a specific machine gathered from events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MachineInfo {
    pub dc_id: Option<String>,
    pub data_hall_id: Option<String>,
    pub zone_id: Option<String>,
    pub machine_id: Option<String>,
    pub class_type: Option<String>,
}

/// Represents the overall simulation report.
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationReport {
    /// The random seed used for the simulation run.
    pub seed: Option<String>,
    /// The total elapsed time reported by the simulation.
    pub elapsed_time: Option<String>,
    /// The total real time reported by the simulation.
    pub real_time: Option<String>,
    /// List of CloggingPair events, sorted by timestamp.
    pub clogging_pairs: Vec<CloggingPairData>,
    /// Summary statistics for CloggingPair events.
    pub clogging_pair_summary: Option<CloggingPairSummary>,
    /// List of ClogInterface events, sorted by timestamp.
    pub clog_interfaces: Vec<ClogInterfaceData>,
    /// Summary statistics for ClogInterface events, grouped by queue name.
    pub clog_interface_summary: HashMap<String, ClogInterfaceSummary>,
    /// List of CoordinatorsChange events, sorted by timestamp.
    pub coordinators_changes: Vec<CoordinatorsChangeData>,
    /// Total count of coordinator changes.
    pub coordinators_change_count: usize,
    /// Details of machines involved in the simulation.
    pub machine_details: HashMap<String, MachineInfo>,
    /// List of DiskSwap events, sorted by timestamp.
    pub disk_swaps: Vec<DiskSwapData>,
    /// List of SetDiskFailure events, sorted by timestamp.
    pub set_disk_failures: Vec<SetDiskFailureData>,
    /// List of CorruptedBlock events, sorted by timestamp.
    pub corrupted_blocks: Vec<CorruptedBlockData>,
}

impl fmt::Display for SimulationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "FoundationDB Simulation Report")?;
        writeln!(f, "==============================")?;

        if let Some(seed) = &self.seed {
            writeln!(f, "Seed: {}", seed)?;
        }
        if let Some(elapsed) = &self.elapsed_time {
            writeln!(f, "Simulated Time: {} seconds", elapsed)?;
        }
        if let Some(real) = &self.real_time {
            writeln!(f, "Real Time: {} seconds", real)?;
        }
        writeln!(f, "")?;

        writeln!(f, "--- Chaos injection Summary ---")?;
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

        writeln!(
            f,
            "  Coordinator Changes: {}",
            self.coordinators_change_count
        )?;
        writeln!(f, "  Disk Swaps: {}", self.disk_swaps.len())?;
        writeln!(f, "  Disk Failures: {}", self.set_disk_failures.len())?;
        writeln!(f, "  Corrupted Blocks: {}", self.corrupted_blocks.len())?;
        writeln!(f, "")?;

        // Initially, don't print the long vectors or detailed maps
        // Add sections here later if needed, possibly behind a flag.
        writeln!(f, "--- Details ---")?;
        writeln!(f, "  Machines Found: {}", self.machine_details.len())?;
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
    let mut elapsed_time = None;
    let mut real_time = None;

    // Use HashMaps to collect unique machine details
    let mut machine_details: HashMap<String, MachineInfo> = HashMap::new();

    // Initialize vectors for other event types
    let mut clogging_pairs = Vec::new();
    let mut clog_interfaces = Vec::new();
    let mut coordinators_changes = Vec::new();
    let mut disk_swaps = Vec::new();
    let mut set_disk_failures = Vec::new();
    let mut corrupted_blocks = Vec::new();
    let mut kill_machine_processes = Vec::new();

    for event in events {
        match event {
            Event::ProgramStart(data) => {
                // Only take the seed from the first ProgramStart event
                if seed.is_none() && data.random_seed.is_some() {
                    seed = data.random_seed.clone();
                }
            }
            Event::ElapsedTime(data) => {
                elapsed_time = Some(data.sim_time.clone());
                real_time = Some(data.real_time.clone());
            }
            Event::CloggingPair(data) => clogging_pairs.push(data.clone()),
            Event::ClogInterface(data) => clog_interfaces.push(data.clone()),
            Event::SimulatedMachineStart(data) => {
                if data.process_class == "test" {
                    continue;
                }

                machine_details.insert(
                    data.machine_id.clone().unwrap(),
                    MachineInfo {
                        dc_id: data.dc_id.clone(),
                        data_hall_id: data.data_hall.clone(),
                        zone_id: data.zone_id.clone(),
                        machine_id: data.machine_id.clone(),
                        class_type: Some(data.process_class.clone()),
                    },
                );
            }
            Event::CoordinatorsChange(data) => coordinators_changes.push(data.clone()),
            Event::DiskSwap(data) => disk_swaps.push(data.clone()),
            Event::SetDiskFailure(data) => set_disk_failures.push(data.clone()),
            Event::CorruptedBlock(data) => corrupted_blocks.push(data.clone()),
            Event::KillMachineProcess(data) => kill_machine_processes.push(data.clone()),
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
    coordinators_changes.sort_by(|a, b| {
        parse_ts(&a.timestamp)
            .partial_cmp(&parse_ts(&b.timestamp))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    disk_swaps.sort_by(|a, b| {
        parse_ts(&a.timestamp)
            .partial_cmp(&parse_ts(&b.timestamp))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    set_disk_failures.sort_by(|a, b| {
        parse_ts(&a.timestamp)
            .partial_cmp(&parse_ts(&b.timestamp))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    corrupted_blocks.sort_by(|a, b| {
        parse_ts(&a.time)
            .partial_cmp(&parse_ts(&b.time))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // --- Calculate Clogging Summary ---
    let mut min_seconds = f64::MAX;
    let mut max_seconds = f64::MIN;
    let mut sum_seconds = 0.0;
    let mut count = 0;

    for pair in &clogging_pairs {
        if let Ok(seconds) = pair.seconds.parse::<f64>() {
            min_seconds = min_seconds.min(seconds);
            max_seconds = max_seconds.max(seconds);
            sum_seconds += seconds;
            count += 1;
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
        None
    };

    // --- Calculate Clog Interface Summary (Grouped by Queue) ---
    let mut interface_stats: HashMap<String, (f64, f64, f64, usize)> = HashMap::new(); // (sum, min, max, count)

    for interface in &clog_interfaces {
        if let Ok(seconds) = interface.delay.parse::<f64>() {
            let queue_name = interface.queue.clone();
            let entry = interface_stats
                .entry(queue_name)
                .or_insert((0.0, f64::MAX, f64::MIN, 0));
            entry.0 += seconds; // sum
            entry.1 = entry.1.min(seconds); // min
            entry.2 = entry.2.max(seconds); // max
            entry.3 += 1; // count
        }
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


    // --- Calculate Coordinator Change Count ---
    let coordinators_change_count = coordinators_changes.len();

    SimulationReport {
        seed,
        elapsed_time,
        real_time,
        clogging_pairs,
        clogging_pair_summary,
        clog_interfaces,
        clog_interface_summary,
        coordinators_changes,
        coordinators_change_count,
        machine_details,
        disk_swaps,
        set_disk_failures,
        corrupted_blocks,
    }
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*; // Import items from outer module (report)
    use crate::parser::{parse_log_file, }; // Import parse_log_file

    use std::path::Path; // Need Path for parse_log_file

    #[test]
    fn test_create_report_from_log() {
        let log_path_str = "logs/combined_trace.0.0.0.0.24.1745498878.p7Loj0.json";
        let log_path = Path::new(log_path_str);

        let events = parse_log_file(log_path).expect(&format!(
            "Failed to parse log file '{}' using parse_log_file",
            log_path.display()
        ));

        let report = create_simulation_report(&events);

        assert_eq!(report.seed, Some("292006968".to_string()));
        assert_eq!(report.elapsed_time, Some("351.752".to_string()));

        assert_eq!(report.clogging_pairs.len(), 396);
        assert!(report.clogging_pair_summary.is_some());
        let clog_pair_summary = report.clogging_pair_summary.as_ref().unwrap();
        assert_eq!(clog_pair_summary.count, 396);
        assert!((clog_pair_summary.min_seconds - 0.000720).abs() < 1e-6);
        assert!((clog_pair_summary.mean_seconds - 0.633275).abs() < 1e-6);
        assert!((clog_pair_summary.max_seconds - 6.198240).abs() < 1e-6);

        assert_eq!(report.clog_interface_summary.len(), 3);

        assert!(report.clog_interface_summary.contains_key("All"));
        let all_summary = report.clog_interface_summary.get("All").unwrap();
        assert_eq!(all_summary.count, 189);
        assert!((all_summary.min_seconds - 0.000051).abs() < 1e-6);
        assert!((all_summary.mean_seconds - 0.263599).abs() < 1e-6);
        assert!((all_summary.max_seconds - 4.536360).abs() < 1e-6);

        assert!(report.clog_interface_summary.contains_key("Receive"));
        let receive_summary = report.clog_interface_summary.get("Receive").unwrap();
        assert_eq!(receive_summary.count, 135);
        assert!((receive_summary.min_seconds - 0.000049).abs() < 1e-6);
        assert!((receive_summary.mean_seconds - 0.325336).abs() < 1e-6);
        assert!((receive_summary.max_seconds - 4.316820).abs() < 1e-6);

        assert!(report.clog_interface_summary.contains_key("Send"));
        let send_summary = report.clog_interface_summary.get("Send").unwrap();
        assert_eq!(send_summary.count, 157);
        assert!((send_summary.min_seconds - 0.000158).abs() < 1e-6);
        assert!((send_summary.mean_seconds - 0.361933).abs() < 1e-6);
        assert!((send_summary.max_seconds - 4.221570).abs() < 1e-6);

        assert_eq!(report.coordinators_changes.len(), 1);
        assert_eq!(report.coordinators_change_count, 1);

        assert_eq!(
            report.machine_details.len(),
            17,
            "Machine details should have 17 entries, got {:?}",
            report.machine_details
        );
        let machine_id_to_check = "25acda3f10d0edab6db5ed5464b34380";
        assert!(report.machine_details.contains_key(machine_id_to_check));
        let machine_info = report.machine_details.get(machine_id_to_check).unwrap();
        assert_eq!(machine_info.dc_id, Some("0".to_string()));
        assert_eq!(machine_info.data_hall_id, Some("0".to_string()));
        assert_eq!(
            machine_info.zone_id,
            Some("0f12bbdbf2c49d14bd0388a344101846".to_string())
        );
        assert_eq!(
            machine_info.machine_id,
            Some(machine_id_to_check.to_string())
        );
        assert_eq!(machine_info.class_type, Some("sim_http_server".to_string()));
    }

    #[test]
    fn test_create_report_with_set_disk_failure() {
        // Create a sample SetDiskFailure event
        let disk_failure_data = SetDiskFailureData {
            timestamp: "150.0".to_string(),
            machine: "1.2.3.4:5".to_string(),
            stall_interval: "10".to_string(),
            stall_period: "10".to_string(),
            stall_until: "160.0".to_string(),
            throttle_period: "60".to_string(),
            throttle_until: "210.0".to_string(),
        };
        let events = vec![Event::SetDiskFailure(disk_failure_data.clone())];

        // Create the report
        let report = create_simulation_report(&events);

        // Assertions
        assert_eq!(report.set_disk_failures.len(), 1);
        assert_eq!(report.set_disk_failures[0], disk_failure_data);
        // Check other fields are default/empty as expected
        assert!(report.seed.is_none());
        assert!(report.elapsed_time.is_none());
        assert!(report.clogging_pairs.is_empty());
        assert!(report.clogging_pair_summary.is_none());
        assert!(report.clog_interfaces.is_empty());
        assert!(report.clog_interface_summary.is_empty());
        assert!(report.coordinators_changes.is_empty());
        assert_eq!(report.coordinators_change_count, 0);
        assert!(report.machine_details.is_empty());
        assert!(report.disk_swaps.is_empty());
        assert!(report.corrupted_blocks.is_empty());
    }
}
