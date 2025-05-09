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

        // --- Left Column Layout ---
        let left_column_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(35), // Top: Combined Overview & Config
                Constraint::Min(10),        // Bottom: Chaos Summary
            ])
            .split(main_columns[0]);

        let top_left_area = left_column_layout[0];
        let overview_config_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Left side: Overview
                Constraint::Percentage(50), // Right side: Config Summary
            ])
            .split(top_left_area);
        let overview_area = overview_config_split[0];
        let config_summary_area = overview_config_split[1];
        let chaos_area = left_column_layout[1];

        // --- Right Column Layout ---
        let right_column_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]) // Top Area | Bottom: Timeline
            .split(main_columns[1]);

        // Split the top-right area vertically for Machine Summary and Process Table
        let top_right_area = right_column_layout[0];
        let distribution_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Fixed height for Machine Summary (adjust as needed)
                Constraint::Min(10),   // Remaining space for Process Table
            ])
            .split(top_right_area);

        // Assign final areas
        let machine_summary_area = distribution_split[0];
        let process_detail_area = distribution_split[1];
        let timeline_area = right_column_layout[1];

        // --- Render Panes ---
        self.render_overview_pane(frame, overview_area);
        self.render_config_summary_pane(frame, config_summary_area);
        self.render_chaos_summary_pane(frame, chaos_area);

        // Render the new distribution panes
        self.render_distribution_panes(frame, machine_summary_area, process_detail_area);

        self.render_timeline_pane(frame, timeline_area);

        // Render Status Bar
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

    /// Renders the content for the "Config Summary" pane.
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

    /// Renders the Machine Distribution summary and Process Distribution table.
    fn render_distribution_panes(
        &self,
        frame: &mut Frame,
        summary_area: Rect, // Area for the DC summary
        table_area: Rect,   // Area for the detailed process table
    ) {
        // --- Data Collection and Processing (Same as before) ---
        let mut machine_list: Vec<(String, String, String, String, String)> = Vec::new();
        let mut dc_counts: HashMap<String, usize> = HashMap::new();

        for machine in self.report.machine_details.values() {
            let class_type = machine
                .class_type
                .clone()
                .unwrap_or_else(|| "unset".to_string());

            if class_type == "sim_http_server" {
                continue;
            }

            let dc_id = machine.dc_id.clone().unwrap_or_else(|| "N/A".to_string());
            let machine_id = machine
                .machine_id
                .clone()
                .unwrap_or_else(|| "N/A".to_string());
            let ip_address = machine
                .ip_address
                .clone()
                .unwrap_or_else(|| "N/A".to_string());
            let process_id = machine
                .machine_id
                .clone()
                .unwrap_or_else(|| "N/A".to_string())
                .split(&['-', '.'])
                .last()
                .unwrap_or("N/A")
                .to_string();

            *dc_counts.entry(dc_id.clone()).or_insert(0) += 1;
            machine_list.push((dc_id, machine_id, ip_address, process_id, class_type));
        }

        machine_list.sort_by(|a, b| {
            let dc_cmp = a.0.cmp(&b.0);
            if dc_cmp == std::cmp::Ordering::Equal {
                a.1.cmp(&b.1)
            } else {
                dc_cmp
            }
        });

        // --- Render Machine Distribution Summary ---
        let summary_block = Block::default()
            .title(Span::styled(
                " Machine Distribution ",
                Style::default().fg(Color::Green),
            ))
            .borders(Borders::ALL);
        let summary_inner_area = summary_block.inner(summary_area);
        frame.render_widget(summary_block, summary_area);

        let mut sorted_dcs: Vec<_> = dc_counts.into_iter().collect();
        sorted_dcs.sort_by(|a, b| a.0.cmp(&b.0));
        let summary_lines: Vec<Line> = sorted_dcs
            .iter()
            .map(|(dc, count)| {
                Line::from(vec![
                    Span::styled(format!("DC {}: ", dc), Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{} machines", count)), // Changed "processes" to "machines"
                ])
            })
            .collect();
        let summary_paragraph = Paragraph::new(summary_lines).wrap(Wrap { trim: true });
        frame.render_widget(summary_paragraph, summary_inner_area);

        // --- Render Process Distribution Table ---
        let table_block = Block::default()
            .title(Span::styled(
                " Process Distribution ",
                Style::default().fg(Color::Green),
            ))
            .borders(Borders::ALL);
        let table_inner_area = table_block.inner(table_area);
        frame.render_widget(table_block, table_area);

        let rows: Vec<Row> = machine_list
            .into_iter()
            .map(|(dc, machine_id, ip_addr, process_id, class)| {
                Row::new(vec![
                    Cell::from(dc),
                    Cell::from(machine_id),
                    Cell::from(ip_addr),
                    Cell::from(process_id),
                    Cell::from(class),
                ])
            })
            .collect();

        let headers = ["DC", "Machine ID", "IP Address", "Process ID", "Class Type"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)))
            .collect::<Row>()
            .height(1)
            .bottom_margin(1);

        let table = Table::new(
            rows,
            [
                Constraint::Length(8),
                Constraint::Length(14),
                Constraint::Length(15),
                Constraint::Length(12),
                Constraint::Min(10),
            ],
        )
        .header(headers)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

        frame.render_widget(table, table_inner_area);
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

            // Extract IP address, handling both IPv4 and IPv6 ([...]:port) and extra info
            let ip_addr = event
                .process // e.g., "... address: 2.1.1.2:1 zone: id ..." or "... address: [::1]:80 zone: id ..."
                .split("address: ")
                .nth(1)
                .map(|addr_part| {
                    // Isolate the ip:port part before the first space (if any)
                    let ip_with_port = addr_part.split(' ').next().unwrap_or(addr_part);

                    if ip_with_port.starts_with('[') && ip_with_port.contains(']') {
                        // Likely IPv6: Extract content within brackets
                        ip_with_port
                            .split('[')
                            .nth(1)
                            .and_then(|s| s.split(']').next())
                            .unwrap_or(ip_with_port) // Fallback
                    } else {
                        // Likely IPv4: Split by last colon to remove port
                        ip_with_port
                            .rsplit_once(':')
                            .map_or(ip_with_port, |(ip, _port)| ip)
                    }
                })
                .unwrap_or("?.?.?.?"); // Default if parsing fails

            // Simplify details format
            let details = format!("{:?} {}", kill_type, ip_addr);
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
