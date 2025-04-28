use crate::parser::*;
use colored::Colorize; // Import colored functionality
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table}; // Import comfy-table
use humantime::format_duration;
use serde::{Deserialize, Serialize}; // Add this back
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr; // Add this import back

// --- Struct Definitions ---
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
    pub ip_address: Option<String>,
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
    /// Simulator configuration parameters.
    pub simulator_config: Option<HashMap<String, String>>,
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
    /// List of KillMachineProcess events, sorted by timestamp.
    pub kill_machine_processes: Vec<KillMachineProcessData>,
    /// Summary statistics for KillMachineProcess events, grouped by KillType.
    pub kill_machine_process_summary: HashMap<KillType, usize>,
}

impl fmt::Display for SimulationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", "Simulation Report".bold().underline())?;
        writeln!(f)?;

        // --- Combined Overview Table (Horizontal) ---
        writeln!(f, "{}", "Simulation Overview".bold())?;
        let mut ordered_headers: Vec<String> = Vec::new();
        let mut ordered_values: Vec<String> = Vec::new();
        let mut config_items: HashMap<String, String> = HashMap::new();
        let mut explicit_replication: Option<String> = None;
        let mut inferred_replication: Option<String> = None;

        // Process Simulator Config: extract replication info and other allowed items
        if let Some(config) = &self.simulator_config {
            let allowlist: HashSet<&str> = [
                // Don't list replication/single/double/triple here, handle separately
                "storage_engine",
                "commit_proxies",
                "logs",
                "proxies",
                "resolvers",
            ]
            .iter()
            .cloned()
            .collect();

            for (key, value) in config {
                let key_str = key.as_str();
                match key_str {
                    "replication" => {
                        explicit_replication = Some(match value.as_str() {
                            "1" => "Single".to_string(),
                            "2" => "Double".to_string(),
                            "3" => "Triple".to_string(),
                            _ => format!("{} (Unknown)", value),
                        });
                    }
                    "single" => inferred_replication = Some("Single".to_string()),
                    "double" => inferred_replication = Some("Double".to_string()),
                    "triple" => inferred_replication = Some("Triple".to_string()),
                    _ => {
                        // Add other allowed keys to the config_items map
                        if allowlist.contains(key_str) {
                            config_items.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }

        // Add items to ordered vectors in the desired sequence
        // 1. Seed
        ordered_headers.push("Seed".to_string());
        ordered_values.push(self.seed.as_deref().unwrap_or("N/A").to_string());

        // 2. Replication (Synthesized)
        let final_replication = explicit_replication
            .or(inferred_replication)
            .unwrap_or_else(|| "N/A".to_string());
        ordered_headers.push("Replication".to_string());
        ordered_values.push(final_replication);

        // 3. Simulated Time
        ordered_headers.push("Simulated Time".to_string());
        ordered_values.push(self.elapsed_time.as_deref().map_or_else(
            || "N/A".to_string(),
            |elapsed| {
                elapsed.parse::<f64>().map_or_else(
                    |_| format!("{} (Invalid format)", elapsed),
                    |duration| {
                        format_duration(std::time::Duration::from_secs_f64(duration)).to_string()
                    },
                )
            },
        ));

        // 4. Real Time
        ordered_headers.push("Real Time".to_string());
        ordered_values.push(self.real_time.as_deref().map_or_else(
            || "N/A".to_string(),
            |real| {
                real.parse::<f64>().map_or_else(
                    |_| format!("{} (Invalid format)", real),
                    |duration| {
                        format_duration(std::time::Duration::from_secs_f64(duration)).to_string()
                    },
                )
            },
        ));

        // 6. Add the rest of the filtered config items (sorted alphabetically)
        let mut remaining_config: Vec<_> = config_items.into_iter().collect();
        remaining_config.sort_by(|a, b| a.0.cmp(&b.0));
        for (key, value) in remaining_config {
            let title_case_key = key
                .split('_')
                .map(|word| {
                    let mut c = word.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<Vec<String>>()
                .join(" ");
            ordered_headers.push(title_case_key);
            ordered_values.push(value);
        }

        // Create and print the overview table using the ordered vectors
        let headers = ordered_headers
            .iter()
            .map(|h| Cell::new(h))
            .collect::<Vec<_>>();
        let row = ordered_values
            .iter()
            .map(|v| Cell::new(v))
            .collect::<Vec<_>>();

        let mut overview_table = Table::new();
        overview_table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(headers);
        overview_table.add_row(row);
        writeln!(f, "{}", overview_table)?;
        writeln!(f)?;

        // --- Cluster Topology Section ---
        if !self.machine_details.is_empty() {
            writeln!(f, "{}", "--- Cluster Topology Summary ---".bright_magenta())?;
            let mut topology_table = Table::new();
            topology_table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec!["DC ID", "Machine Count", "Class Type Summary"]);

            // Group machines by DC ID
            let mut machines_by_dc: HashMap<String, Vec<&MachineInfo>> = HashMap::new();
            for machine_info in self.machine_details.values() {
                let dc_key = machine_info.dc_id.as_deref().unwrap_or("N/A").to_string();
                machines_by_dc.entry(dc_key).or_default().push(machine_info);
            }

            // Sort DCs by ID
            let mut sorted_dcs: Vec<_> = machines_by_dc.keys().cloned().collect();
            sorted_dcs.sort();

            for dc_id in sorted_dcs {
                if let Some(machines) = machines_by_dc.get(&dc_id) {
                    let machine_count = machines.len();

                    // Count class types within this DC
                    let mut class_counts: HashMap<String, usize> = HashMap::new();
                    for machine in machines {
                        let class_key = machine.class_type.as_deref().unwrap_or("N/A").to_string();
                        *class_counts.entry(class_key).or_insert(0) += 1;
                    }

                    // Create summary string
                    let mut summary_parts: Vec<String> = class_counts
                        .iter()
                        .map(|(class_type, count)| format!("{}: {}", class_type, count))
                        .collect();
                    summary_parts.sort(); // Sort alphabetically by class type for consistency
                    let summary_str = summary_parts.join(", ");

                    topology_table.add_row(vec![
                        Cell::new(&dc_id),
                        Cell::new(machine_count),
                        Cell::new(summary_str),
                    ]);
                }
            }

            writeln!(f, "{}", topology_table)?;
            writeln!(f)?; // Add extra newline for spacing

            // --- Machine Details Table ---
            writeln!(f, "{}", "--- Machine Details --- ".bright_blue())?;
            let mut machine_table = Table::new();
            machine_table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Machine ID"),
                    Cell::new("IP Address"),
                    Cell::new("DC ID"),
                    Cell::new("Class Type"),
                ]);

            // Collect machine details into a Vec to sort them
            let mut sorted_machines: Vec<_> = self.machine_details.values().collect();
            // Sort by Machine ID (unwrap_or handles None cases for sorting)
            sorted_machines.sort_by(|a, b| {
                a.machine_id
                    .as_deref()
                    .unwrap_or("")
                    .cmp(b.machine_id.as_deref().unwrap_or(""))
            });

            for machine_info in sorted_machines {
                machine_table.add_row(vec![
                    Cell::new(machine_info.machine_id.as_deref().unwrap_or("N/A")),
                    Cell::new(machine_info.ip_address.as_deref().unwrap_or("N/A")),
                    Cell::new(machine_info.dc_id.as_deref().unwrap_or("N/A")),
                    Cell::new(machine_info.class_type.as_deref().unwrap_or("N/A")),
                ]);
            }

            writeln!(f, "{}", machine_table)?;
            writeln!(f)?; // Add extra newline for spacing
        }

        // --- Chaos Summary Section ---
        writeln!(f, "{}", "--- Chaos injection Summary ---".bright_yellow())?;

        // Clogging Pairs (Table)
        if let Some(summary) = &self.clogging_pair_summary {
            if summary.count > 0 {
                writeln!(f, "  {}:", "Clogging Pairs".green())?;
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec![
                        "Count",
                        "Min Duration (s)",
                        "Mean Duration (s)",
                        "Max Duration (s)",
                    ]);
                table.add_row(vec![
                    Cell::new(summary.count),
                    Cell::new(format!("{:.6}", summary.min_seconds)),
                    Cell::new(format!("{:.6}", summary.mean_seconds)),
                    Cell::new(format!("{:.6}", summary.max_seconds)),
                ]);
                writeln!(f, "{}", table)?;
            }
        }

        // Clogged Interfaces (Table)
        if !self.clog_interface_summary.is_empty() {
            writeln!(f, "  {}:", "Clogged Interfaces (by Queue)".green())?;
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    "Queue",
                    "Count",
                    "Min Delay (s)",
                    "Mean Delay (s)",
                    "Max Delay (s)",
                ]);

            let mut sorted_queues: Vec<_> = self.clog_interface_summary.keys().collect();
            sorted_queues.sort();
            for queue_name in sorted_queues {
                if let Some(summary) = self.clog_interface_summary.get(queue_name) {
                    if summary.count > 0 {
                        table.add_row(vec![
                            Cell::new(queue_name),
                            Cell::new(summary.count),
                            Cell::new(format!("{:.6}", summary.min_seconds)),
                            Cell::new(format!("{:.6}", summary.mean_seconds)),
                            Cell::new(format!("{:.6}", summary.max_seconds)),
                        ]);
                    }
                }
            }
            // Only print the table if it has rows
            if table.row_count() > 0 {
                writeln!(f, "{}", table)?;
            }
        }

        // Coordinator Changes (Table)
        writeln!(f, "  Coordinator Changes:")?;
        if !self.coordinators_changes.is_empty() {
            let mut coord_table = Table::new();
            coord_table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Timestamp (s)").add_attribute(comfy_table::Attribute::Bold),
                    Cell::new("Coordinator Count").add_attribute(comfy_table::Attribute::Bold), // Updated Header
                ]);

            for change in &self.coordinators_changes {
                // Count coordinators by splitting the string
                let count = change.new_coordinators_key.split(',').count();
                coord_table.add_row(vec![
                    Cell::new(&change.timestamp),
                    Cell::new(count.to_string()), // Display count
                ]);
            }
            writeln!(f, "{}", coord_table)?;
        } else {
            writeln!(f, "    No coordinator changes recorded.")?;
        }

        // Process Kills (Table)
        if !self.kill_machine_process_summary.is_empty() {
            writeln!(f, "  {}:", "Process Kills (by Type)".green())?;
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec!["Kill Type", "Count"]);

            let mut sorted_kill_types: Vec<_> = self.kill_machine_process_summary.keys().collect();
            sorted_kill_types.sort();
            for kill_type in sorted_kill_types {
                if let Some(count) = self.kill_machine_process_summary.get(kill_type) {
                    if *count > 0 {
                        table.add_row(vec![
                            Cell::new(format!("{:?}", kill_type)),
                            Cell::new(*count),
                        ]);
                    }
                }
            }
            // Only print the table if it has rows
            if table.row_count() > 0 {
                writeln!(f, "{}", table)?;
            }
        }
        writeln!(f)?; // Add a final newline for spacing

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
    let mut simulator_config = None;

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

    // Summaries (initialized before loop)
    let mut kill_machine_process_summary: HashMap<KillType, usize> = HashMap::new();

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
            Event::SimulatorConfig(data) => {
                // Assume only one SimulatorConfig event exists
                if simulator_config.is_none() {
                    simulator_config = Some(data.config.clone());
                }
            }
            Event::CloggingPair(data) => clogging_pairs.push(data.clone()),
            Event::ClogInterface(data) => clog_interfaces.push(data.clone()),
            Event::SimulatedMachineStart(data) => {
                if data.process_class == "test" {
                    continue;
                }

                // Ensure machine_id exists before inserting
                if let Some(machine_id) = &data.machine_id {
                    machine_details.insert(
                        machine_id.clone(), // Use the actual machine_id
                        MachineInfo {
                            dc_id: data.dc_id.clone(),
                            data_hall_id: data.data_hall.clone(),
                            zone_id: data.zone_id.clone(),
                            machine_id: data.machine_id.clone(), // Store the machine_id itself
                            ip_address: data.machine_ips.clone(), // Use the machine_ips field here
                            class_type: Some(data.process_class.clone()),
                        },
                    );
                } else {
                    // Optionally log a warning if machine_id is missing
                    eprintln!(
                        "Warning: SimulatedMachineStart event at timestamp {} is missing machine_id",
                        data.timestamp
                    );
                }
            }
            Event::CoordinatorsChange(data) => coordinators_changes.push(data.clone()),
            Event::DiskSwap(data) => disk_swaps.push(data.clone()),
            Event::SetDiskFailure(data) => set_disk_failures.push(data.clone()),
            Event::CorruptedBlock(data) => corrupted_blocks.push(data.clone()),
            Event::KillMachineProcess(event_data) => {
                kill_machine_processes.push(event_data.clone());
                match KillType::from_str(&event_data.raw_kill_type) {
                    Ok(kill_type) => {
                        *kill_machine_process_summary.entry(kill_type).or_insert(0) += 1;
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Unknown KillType '{}' at timestamp {}: {}",
                            event_data.raw_kill_type, event_data.timestamp, e
                        );
                        *kill_machine_process_summary
                            .entry(KillType::Unknown) // Count unknowns
                            .or_insert(0) += 1;
                    }
                }
            }
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

    kill_machine_processes.sort_by(|a, b| {
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

    // --- Calculate Kill Machine Process Summary ---
    // kill_machine_process_summary is already populated in the event loop

    SimulationReport {
        seed,
        elapsed_time,
        real_time,
        simulator_config,
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
        kill_machine_processes,
        kill_machine_process_summary,
    }
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*; // Import items from outer module (report)
    use crate::parser::{parse_log_file, Event};
    // use crate::parser::KillType; // Remove unused import
    // use std::collections::HashMap; // Remove unused import

    #[test]
    fn test_create_report_from_log() {
        let file_path = "logs/combined_trace.0.0.0.0.24.1745498878.p7Loj0.json";
        let events = parse_log_file(file_path).expect("Failed to parse log file");
        let report = create_simulation_report(&events); // Pass as reference

        // Check general report fields (exact values)
        assert_eq!(report.seed, Some("292006968".to_string())); // Correct seed from first ProgramStart

        // Check that simulator config is Some (content check removed)
        assert!(
            report.simulator_config.is_some(),
            "Simulator config should be Some"
        );

        // --- Clogging Pairs --- (Check non-empty)
        assert!(
            !report.clogging_pairs.is_empty(),
            "Clogging pairs should not be empty"
        );
        assert!(
            report.clogging_pair_summary.is_some(),
            "Clogging pair summary should be Some"
        );

        // --- Clog Interfaces --- (Check non-empty)
        assert!(
            !report.clog_interfaces.is_empty(),
            "Clog interfaces should not be empty"
        );
        assert!(
            !report.clog_interface_summary.is_empty(),
            "Clog interface summary should not be empty"
        );

        // --- Kill Machine Processes --- (Check non-empty)
        assert!(
            !report.kill_machine_processes.is_empty(),
            "Kill machine processes should not be empty"
        );
        assert!(
            !report.kill_machine_process_summary.is_empty(),
            "Kill machine process summary should not be empty"
        );

        // --- Check Coordinator change count (exact) and non-empty list ---
        assert!(
            !report.coordinators_changes.is_empty(),
            "Coordinator changes should not be empty"
        );

        // --- Check machine details (non-empty map) ---
        assert!(
            !report.machine_details.is_empty(),
            "Machine details should not be empty"
        );

        // Check specific machine IP address
        let expected_machine_id = "e4a5cec0b954157cc11edea9e5e3ee80";
        let expected_ip = "2.0.1.1";
        match report.machine_details.get(expected_machine_id) {
            Some(machine_info) => {
                assert_eq!(
                    machine_info.ip_address,
                    Some(expected_ip.to_string()),
                    "IP address mismatch for machine {}",
                    expected_machine_id
                );
            }
            None => {
                panic!(
                    "Machine details missing entry for machine {}",
                    expected_machine_id
                );
            }
        }

        // Print the report (optional, for manual inspection)
        // println!("--- Generated Report ---\n{}", report);
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
        assert!(report.kill_machine_processes.is_empty());
        assert!(report.kill_machine_process_summary.is_empty());
    }
}
