use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};
use sysinfo::System;

use crate::monitor::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title and system info
            Constraint::Min(0),     // Processes table
            Constraint::Length(3),  // Help text
        ])
        .split(f.size());

    // Draw title and system info
    let system_info = format!(
        "CPU: {:.1}% | Memory: {:.1}MB / {:.1}MB",
        app.system.global_cpu_info().cpu_usage(),
        app.system.used_memory() / 1024 / 1024,
        app.system.total_memory() / 1024 / 1024
    );
    
    let title = Paragraph::new(Text::styled(
        "RustyWatch Process Monitor",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    ))
    .block(Block::default().borders(Borders::BOTTOM))
    .alignment(Alignment::Center);
    
    f.render_widget(title, chunks[0]);
    
    let sys_info = Paragraph::new(Text::styled(
        system_info,
        Style::default().fg(Color::White),
    ))
    .alignment(Alignment::Center);
    
    f.render_widget(sys_info, chunks[0]);

    // Draw process table
    let header_cells = ["PID", "Name", "CPU%", "Memory (MB)", "Status"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray))
        .height(1);

    // Get processes and sort by CPU usage
    let mut processes = app.system.processes()
        .iter()
        .collect::<Vec<_>>();
    
    processes.sort_by(|a, b| {
        let (_, proc_a) = a;
        let (_, proc_b) = b;
        proc_b.cpu_usage().partial_cmp(&proc_a.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // Create rows from processes
    let rows = processes.iter().map(|(pid, proc)| {
        let cells = [
            Cell::from(format!("{}", pid)),
            Cell::from(proc.name()),
            Cell::from(format!("{:.1}%", proc.cpu_usage())),
            Cell::from(format!("{} MB", proc.memory() / 1024 / 1024)),
            Cell::from(format!("{:?}", proc.status())),
        ];
        Row::new(cells).height(1)
    });

    let process_table = Table::new(
        rows,
        [
            Constraint::Length(10),  // PID
            Constraint::Percentage(40), // Name
            Constraint::Length(10),  // CPU%
            Constraint::Length(13),  // Memory
            Constraint::Length(10),  // Status
        ]
    )
        .header(header)
        .block(
            Block::default()
                .title("Process Information")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .column_spacing(1)
        .highlight_style(Style::default().bg(Color::Blue));

    f.render_widget(process_table, chunks[1]);

    // Draw help text
    let help_text = Paragraph::new(Text::styled(
        "Press [q] to quit | [r] to refresh",
        Style::default().fg(Color::Gray),
    ))
    .block(Block::default().borders(Borders::TOP))
    .alignment(Alignment::Center);

    f.render_widget(help_text, chunks[2]);
}
