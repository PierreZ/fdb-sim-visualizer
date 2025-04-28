use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind};
use humantime::format_duration;
use parser::parser::KillType;
use parser::report::SimulationReport;
use ratatui::{prelude::*, widgets::*};
use std::{
    collections::HashMap,
    io::{self, Write},
    str::FromStr,
    time::{Duration, Instant},
};

/// Represents the main application state.
pub struct App {
    /// The simulation report data.
    report: SimulationReport,
    /// Flag to control application exit.
    should_quit: bool,
    // Add state for scrolling within panes later if needed
    // e.g., overview_scroll: u16, topology_scroll: u16, etc.
}

impl App {
    /// Creates a new application instance.
    pub fn new(report: SimulationReport) -> Self {
        Self {
            report,
            should_quit: false,
            // Initialize scroll states here if added
        }
    }

    /// Runs the main application loop.
    pub fn run(&mut self, terminal: &mut Terminal<impl Backend + Write>) -> io::Result<()> {
        // (Main loop remains the same)
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|frame| self.ui(frame))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let CrosstermEvent::Key(key) = event::read()? {
                    self.handle_key_event(key)?;
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }

    /// Handles key press events (only exit for now).
    fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.should_quit = true;
                }
                // TODO: Add keys for scrolling within focused panes (e.g., Up/Down/PgUp/PgDown)
                // TODO: Add keys for switching focus between panes (e.g., Arrow keys, Tab)
                _ => {}
            }
        }
        Ok(())
    }

    /// Renders the user interface with a split-pane layout.
    fn ui(&self, frame: &mut Frame) {
        // Define outer layout for status bar
        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(95), // Main area for panes
                Constraint::Length(1),      // Status bar at the bottom
            ])
            .split(frame.size());

        // Define the main horizontal split into two columns
        let main_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]) // Left | Right
            .split(outer_layout[0]);

        // Split the Left column vertically into three parts
        let left_column_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20), // Top: Overview
                Constraint::Length(8),      // Middle: Config Summary (fixed height like before)
                Constraint::Min(10),        // Bottom: Chaos Summary
            ])
            .split(main_columns[0]);

        // Split the Right column vertically into two parts
        let right_column_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]) // Top: Machine Distribution | Bottom: Timeline
            .split(main_columns[1]);

        // Assign areas
        let overview_area = left_column_layout[0];
        let config_summary_area = left_column_layout[1];
        let chaos_area = left_column_layout[2];
        let machine_distribution_area = right_column_layout[0];
        let timeline_area = right_column_layout[1];

        // Render each pane in its designated area
        self.render_overview_pane(frame, overview_area);
        self.render_config_summary_pane(frame, config_summary_area);
        self.render_chaos_summary_pane(frame, chaos_area);
        self.render_topology_pane(frame, machine_distribution_area); // Renamed 'render_topology_pane' now handles Machine Distribution
        self.render_timeline_pane(frame, timeline_area);

        // Render Status Bar at the bottom
        self.render_status_bar(frame, outer_layout[1]);
    }

    /// Renders the content for the "Overview" pane.
    fn render_overview_pane(&self, frame: &mut Frame, area: Rect) {
        let overview_block = Block::default()
            .title(Span::styled(
                " Overview ",
                Style::default().fg(Color::Green),
            ))
            .borders(Borders::ALL);
        let inner_area = overview_block.inner(area);

        let mut overview_items: Vec<ListItem> = Vec::new();

        // Seed
        overview_items.push(ListItem::new(Line::from(vec![
            Span::styled("Seed:             ", Style::default().fg(Color::Yellow)),
            Span::raw(self.report.seed.as_deref().unwrap_or("N/A")),
        ])));

        // Simulated Time
        let sim_time_str = self.report.elapsed_time.as_deref().map_or_else(
            || "N/A".to_string(),
            |t| {
                t.parse::<f64>().map_or(format!("{} (Invalid)", t), |d| {
                    format_duration(Duration::from_secs_f64(d)).to_string()
                })
            },
        );
        overview_items.push(ListItem::new(Line::from(vec![
            Span::styled("Simulated Time:   ", Style::default().fg(Color::Yellow)),
            Span::styled(sim_time_str, Style::default().fg(Color::Cyan)),
        ])));

        // Real Time
        let real_time_str = self.report.real_time.as_deref().map_or_else(
            || "N/A".to_string(),
            |t| {
                t.parse::<f64>().map_or(format!("{} (Invalid)", t), |d| {
                    format_duration(Duration::from_secs_f64(d)).to_string()
                })
            },
        );
        overview_items.push(ListItem::new(Line::from(vec![
            Span::styled("Real Time:        ", Style::default().fg(Color::Yellow)),
            Span::styled(real_time_str, Style::default().fg(Color::Cyan)),
        ])));

        let overview_list = List::new(overview_items)
            .block(overview_block)
            .style(Style::default().fg(Color::White));

        // Render the list within the inner area
        frame.render_widget(overview_list, inner_area); // Render list inside the block
    }

    /// Renders the content for the "Topology" pane.
    fn render_topology_pane(&self, frame: &mut Frame, area: Rect) {
        let topology_block = Block::default()
            .title(Span::styled(
                " Process Distribution ",
                Style::default().fg(Color::Green),
            ))
            .borders(Borders::ALL);

        // 1. Filter out sim_http_server and collect machine details
        let mut machine_list: Vec<(String, String, String, String)> = Vec::new();

        for machine in self.report.machine_details.values() {
            let class_type = machine
                .class_type
                .clone()
                .unwrap_or_else(|| "unset".to_string());

            // Skip sim_http_server entirely
            if class_type == "sim_http_server" {
                continue;
            }

            let dc = machine
                .dc_id
                .clone()
                .unwrap_or_else(|| "Unknown".to_string());
            let machine_id = machine
                .data_hall_id
                .clone()
                .unwrap_or_else(|| "N/A".to_string()); // Use data_hall_id
            let process_id = machine.zone_id.clone().unwrap_or_else(|| "N/A".to_string()); // Use zone_id

            machine_list.push((dc, machine_id, process_id, class_type));
        }

        // 2. Prepare data for the table, sorted
        // Sort by DC, MachineID, ProcessID, then Class Type
        machine_list.sort();

        // 3. Define Header
        let header_cells = [
            Cell::from("DC").style(Style::default().fg(Color::Yellow)),
            Cell::from("Machine ID").style(Style::default().fg(Color::Yellow)), // Renamed from Rack
            Cell::from("Process ID").style(Style::default().fg(Color::Yellow)), // Renamed from ServerID
            Cell::from("Class Type").style(Style::default().fg(Color::Yellow)),
        ];
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        // 4. Create Rows
        let rows: Vec<Row> = machine_list
            .into_iter()
            .map(|(dc, machine_id, process_id, class_type)| {
                let cells = [
                    Cell::from(dc),
                    Cell::from(machine_id),
                    Cell::from(process_id),
                    Cell::from(Span::styled(
                        class_type,
                        Style::default().fg(Color::Magenta),
                    )),
                ];
                Row::new(cells).height(1)
            })
            .collect();

        // 5. Define Column Widths
        let widths = [
            Constraint::Length(8),  // DC
            Constraint::Length(12), // Machine ID
            Constraint::Length(12), // Process ID
            Constraint::Max(25),    // Class Type (flexible)
        ];

        // 6. Create and Render Table
        let machine_table = Table::new(rows, widths)
            .header(header)
            .block(topology_block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(machine_table, area);
    }

    /// Renders the content for the "Chaos Summary" pane.
    fn render_chaos_summary_pane(&self, frame: &mut Frame, area: Rect) {
        // Define a two-column layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // --- Clogging Pairs ---
        let clogging_pairs_block = Block::default()
            .title(Span::styled(
                "Network splits",
                Style::default().fg(Color::Green),
            ))
            .borders(Borders::ALL);
        let mut clogging_pairs_text = Vec::new();
        if let Some(summary) = &self.report.clogging_pair_summary {
            clogging_pairs_text.push(Line::from(format!("Count: {}", summary.count)));
            clogging_pairs_text.push(Line::from(format!(
                "  Min Duration:   {}",
                format_duration(Duration::from_secs_f64(summary.min_seconds))
            )));
            clogging_pairs_text.push(Line::from(format!(
                "  Mean Duration:  {}",
                format_duration(Duration::from_secs_f64(summary.mean_seconds))
            )));
            clogging_pairs_text.push(Line::from(format!(
                "  Max Duration:   {}",
                format_duration(Duration::from_secs_f64(summary.max_seconds))
            )));
        } else {
            clogging_pairs_text.push(Line::from("No clogging pairs reported."));
        }

        let clogging_pairs_paragraph = Paragraph::new(clogging_pairs_text)
            .block(clogging_pairs_block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(clogging_pairs_paragraph, chunks[0]);

        // --- Clogged Interfaces ---
        let clogged_interfaces_block = Block::default()
            .title(Span::styled(
                "Network latencies",
                Style::default().fg(Color::Green),
            ))
            .borders(Borders::ALL);
        let mut clogged_interface_items: Vec<ListItem> = Vec::new();

        if !self.report.clog_interface_summary.is_empty() {
            let mut sorted_interfaces: Vec<_> = self.report.clog_interface_summary.iter().collect();
            sorted_interfaces.sort_by(|a, b| a.0.cmp(b.0)); // Sort by queue name

            for (queue_name, summary) in sorted_interfaces {
                // Add blank line before the next queue header, except for the first one
                if !clogged_interface_items.is_empty() {
                    clogged_interface_items.push(ListItem::new(""));
                }

                // Add a header for the queue name
                clogged_interface_items.push(ListItem::new(Line::from(Span::styled(
                    queue_name,
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Yellow),
                ))));
                clogged_interface_items.push(ListItem::new(format!("  Count: {}", summary.count)));
                clogged_interface_items.push(ListItem::new(format!(
                    "    Min Duration:  {}",
                    format_duration(Duration::from_secs_f64(summary.min_seconds))
                )));
                clogged_interface_items.push(ListItem::new(format!(
                    "    Mean Duration: {}",
                    format_duration(Duration::from_secs_f64(summary.mean_seconds))
                )));
                clogged_interface_items.push(ListItem::new(format!(
                    "    Max Duration:  {}",
                    format_duration(Duration::from_secs_f64(summary.max_seconds))
                )));
            }
        } else {
            clogged_interface_items.push(ListItem::new("No clogged interfaces reported."));
        }

        let clogged_interfaces_list = List::new(clogged_interface_items)
            .block(clogged_interfaces_block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(clogged_interfaces_list, chunks[1]);
    }

    /// Renders the content for the "Timeline" pane. (Placeholder)
    fn render_timeline_pane(&self, frame: &mut Frame, area: Rect) {
        let timeline_block = Block::default()
            .title(Span::styled(
                " Timeline ",
                Style::default().fg(Color::Green),
            ))
            .borders(Borders::ALL);

        // Use a Table widget for better alignment
        let header_cells = ["Time (s)", "Event", "Details"].iter().map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::DarkGray))
            .height(1)
            .bottom_margin(1);

        // Define column widths
        let widths = [
            Constraint::Length(10), // Fixed width for time
            Constraint::Length(15), // Fixed width for event type
            Constraint::Min(30),    // Minimum width for details, expands
        ];

        #[derive(Debug)]
        struct TimelineEvent {
            timestamp: f64,
            event_type: String,
            details: String,
        }

        let mut timeline_events: Vec<TimelineEvent> = Vec::new();

        // Helper to parse timestamp and add event
        let mut add_event = |timestamp_str: &str, event_type: String, details: String| {
            if let Ok(ts) = f64::from_str(timestamp_str) {
                timeline_events.push(TimelineEvent {
                    timestamp: ts,
                    event_type,
                    details,
                });
            } else {
                // Log or handle parse error if needed
                eprintln!(
                    "Warning: Could not parse timestamp '{}' for timeline",
                    timestamp_str
                );
            }
        };

        // 1. Coordinator Changes
        for event in &self.report.coordinators_changes {
            let details = "Triggering leader election".to_string();
            add_event(&event.timestamp, "Coord Change".to_string(), details);
        }

        // 2. Killed Processes
        for event in &self.report.kill_machine_processes {
            // Parse KillType
            let kill_type = KillType::from_str(&event.raw_kill_type).unwrap_or(KillType::Unknown);

            // Extract IP address (best effort based on observed format)
            let ip_addr = event
                .process
                .split("address: ")
                .nth(1)
                .and_then(|addr_part| addr_part.split(':').next())
                .unwrap_or("?.?.?.?"); // Default if parsing fails

            let details = format!("Killed (Type: {:?}, IP: {})", kill_type, ip_addr);
            add_event(&event.timestamp, "Reboot".to_string(), details);
        }

        // 3. Disk Swaps
        for event in &self.report.disk_swaps {
            let details = format!("IPs: {}", event.machine_ips);
            add_event(&event.timestamp, "Disk Swap".to_string(), details);
        }

        // Sort events chronologically
        timeline_events.sort_by(|a, b| {
            a.timestamp
                .partial_cmp(&b.timestamp)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Create table rows from events
        let rows: Vec<Row> = timeline_events
            .iter()
            .map(|event| {
                let time_str = format!("{:.3}", event.timestamp);
                Row::new(vec![
                    Cell::from(Span::styled(time_str, Style::default().fg(Color::Cyan))),
                    Cell::from(Span::styled(
                        event.event_type.clone(),
                        Style::default().fg(Color::Magenta),
                    )),
                    Cell::from(event.details.clone()),
                ])
            })
            .collect();

        // Create the table
        let timeline_table = Table::new(rows, widths)
            .block(timeline_block)
            .header(header)
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED)) // Optional: for selection
            .highlight_symbol(">> ") // Optional: for selection
            .style(Style::default().fg(Color::White));

        frame.render_widget(timeline_table, area);
    }

    /// Renders the content for the "Config Summary" sub-pane within Topology.
    fn render_config_summary_pane(&self, frame: &mut Frame, area: Rect) {
        let config_block = Block::default()
            .title(Span::styled(
                " Config Summary ",
                Style::default().fg(Color::Green),
            ))
            .borders(Borders::ALL);

        let mut config_items: Vec<Line> = Vec::new();

        if let Some(config) = &self.report.simulator_config {
            // Determine Replication Mode
            let replication_mode = config
                .get("replication")
                .map(|s| s.as_str())
                .or_else(
                    || match config.get("logs").and_then(|s| s.parse::<i32>().ok()) {
                        Some(1) => Some("single"),
                        Some(3) => Some("double"),
                        Some(5) => Some("triple"),
                        _ => None, // Could be other values or not present
                    },
                )
                .unwrap_or("unknown"); // Default if neither key helps

            config_items.push(Line::from(format!("Replication: {}", replication_mode)));

            // Display other key config values
            let keys_to_show = [
                "storage_engine",
                "commit_proxies",
                "logs",
                "proxies",
                "resolvers",
            ];
            for key in keys_to_show {
                let value = config.get(key).map(|s| s.as_str()).unwrap_or("N/A");
                // Capitalize first letter for display
                let display_key = key
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

                config_items.push(Line::from(format!("{}: {}", display_key, value)));
            }
        } else {
            config_items.push(Line::from("Simulator config not found."));
        }

        // Add padding/empty line at the end if desired
        // config_items.push(Line::from("")); // Makes it look less cramped

        let config_paragraph = Paragraph::new(config_items)
            .block(config_block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(config_paragraph, area);
    }

    /// Renders a simple status bar at the bottom.
    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let status_text = "Quit: q";
        let status_paragraph = Paragraph::new(status_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Right);
        frame.render_widget(status_paragraph, area);
    }
} // End of impl App

/// Sets up the terminal for TUI interaction.
pub fn setup_terminal() -> io::Result<Terminal<impl Backend + Write>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restores the terminal to its original state.
pub fn restore_terminal(terminal: &mut Terminal<impl Backend + Write>) -> io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()
}
