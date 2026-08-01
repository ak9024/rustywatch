use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Margin, Rect},
    prelude::*,
    symbols,
    widgets::{
        Block, BorderType, Borders, Cell, Clear, LineGauge, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Sparkline, Table, Wrap,
    },
};

use crate::monitor::app::{App, AppMode, ConfirmAction, SortColumn, SortOrder};
use crate::monitor::theme::Theme;

pub fn draw(f: &mut Frame, app: &mut App) {
    let theme = Theme::default();
    let size = f.size();

    // Responsive layout based on terminal size
    if size.width >= 120 && size.height >= 30 {
        draw_full_layout(f, app, &theme);
    } else if size.width >= 80 {
        draw_compact_layout(f, app, &theme);
    } else {
        draw_minimal_layout(f, app, &theme);
    }

    // Draw overlays on top
    match app.mode {
        AppMode::Help => draw_help_overlay(f, &theme),
        AppMode::Confirm(action) => draw_confirm_dialog(f, &theme, action),
        _ => {}
    }
}

/// Full layout for large terminals (120+ columns)
fn draw_full_layout(f: &mut Frame, app: &mut App, theme: &Theme) {
    let show_search = !app.search_query.is_empty() || matches!(app.mode, AppMode::Search);

    let constraints = if show_search {
        vec![
            Constraint::Length(3), // Header
            Constraint::Length(3), // Gauges row
            Constraint::Length(5), // Sparklines row
            Constraint::Length(3), // Search bar
            Constraint::Min(10),   // Process table
            Constraint::Length(3), // Footer
        ]
    } else {
        vec![
            Constraint::Length(3), // Header
            Constraint::Length(3), // Gauges row
            Constraint::Length(5), // Sparklines row
            Constraint::Min(10),   // Process table
            Constraint::Length(3), // Footer
        ]
    };

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(f.size());

    draw_header(f, app, theme, main_chunks[0]);

    // Split gauge row into two columns
    let gauge_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    draw_cpu_gauge(f, app, theme, gauge_chunks[0]);
    draw_memory_gauge(f, app, theme, gauge_chunks[1]);

    // Split sparkline row into two columns
    let sparkline_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[2]);

    draw_cpu_sparkline(f, app, theme, sparkline_chunks[0]);
    draw_memory_sparkline(f, app, theme, sparkline_chunks[1]);

    if show_search {
        draw_search_bar(f, app, theme, main_chunks[3]);
        draw_process_table(f, app, theme, main_chunks[4]);
        draw_footer(f, app, theme, main_chunks[5]);
    } else {
        draw_process_table(f, app, theme, main_chunks[3]);
        draw_footer(f, app, theme, main_chunks[4]);
    }
}

/// Compact layout for medium terminals (80-119 columns)
fn draw_compact_layout(f: &mut Frame, app: &mut App, theme: &Theme) {
    let show_search = !app.search_query.is_empty() || matches!(app.mode, AppMode::Search);

    let constraints = if show_search {
        vec![
            Constraint::Length(3), // Header with stats
            Constraint::Length(3), // Search bar
            Constraint::Min(10),   // Process table
            Constraint::Length(2), // Footer
        ]
    } else {
        vec![
            Constraint::Length(3), // Header with stats
            Constraint::Min(10),   // Process table
            Constraint::Length(2), // Footer
        ]
    };

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(f.size());

    draw_header_with_stats(f, app, theme, main_chunks[0]);

    if show_search {
        draw_search_bar(f, app, theme, main_chunks[1]);
        draw_process_table(f, app, theme, main_chunks[2]);
        draw_footer_compact(f, app, theme, main_chunks[3]);
    } else {
        draw_process_table(f, app, theme, main_chunks[1]);
        draw_footer_compact(f, app, theme, main_chunks[2]);
    }
}

/// Minimal layout for small terminals (<80 columns)
fn draw_minimal_layout(f: &mut Frame, app: &mut App, theme: &Theme) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title only
            Constraint::Min(5),    // Process table
            Constraint::Length(1), // Help only
        ])
        .split(f.size());

    let title = Paragraph::new("RustyWatch")
        .style(theme.title_style())
        .alignment(Alignment::Center);
    f.render_widget(title, main_chunks[0]);

    draw_process_table_minimal(f, app, theme, main_chunks[1]);

    let help = Paragraph::new("[q]uit [r]efresh [j/k]nav [?]help")
        .style(Style::default().fg(theme.muted))
        .alignment(Alignment::Center);
    f.render_widget(help, main_chunks[2]);
}

fn draw_header(f: &mut Frame, _app: &App, theme: &Theme, area: Rect) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme.border_style())
        .border_type(BorderType::Rounded);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("RustyWatch", theme.title_style()),
        Span::styled(" Process Monitor ", Style::default().fg(theme.muted)),
        Span::styled(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(theme.accent),
        ),
    ]))
    .block(block)
    .alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn draw_header_with_stats(f: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let cpu = app.system.global_cpu_info().cpu_usage();
    let mem_percent = (app.system.used_memory() as f64 / app.system.total_memory() as f64) * 100.0;

    let header = Paragraph::new(Line::from(vec![
        Span::styled("RustyWatch", theme.title_style()),
        Span::styled(" | ", Style::default().fg(theme.border)),
        Span::styled("CPU: ", Style::default().fg(theme.muted)),
        Span::styled(
            format!("{:.1}%", cpu),
            Style::default().fg(theme.status_color(cpu, 50.0, 80.0)),
        ),
        Span::styled(" | ", Style::default().fg(theme.border)),
        Span::styled("MEM: ", Style::default().fg(theme.muted)),
        Span::styled(
            format!("{:.1}%", mem_percent),
            Style::default().fg(theme.status_color(mem_percent as f32, 60.0, 85.0)),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(theme.border_style()),
    )
    .alignment(Alignment::Center);

    f.render_widget(header, area);
}

/// Converts a 0-100 percentage into the 0.0-1.0 ratio `LineGauge` demands.
///
/// `LineGauge::ratio` panics outside that range, so `NaN` (from a zero total)
/// and out-of-range readings are normalized here rather than taking the whole
/// TUI down while the terminal is in raw mode.
fn gauge_ratio(percentage: f64) -> f64 {
    if percentage.is_nan() {
        0.0
    } else {
        (percentage / 100.0).clamp(0.0, 1.0)
    }
}

/// Percentage of `total` used by `used`, or `0.0` when `total` is unknown.
fn used_percentage(used: f64, total: f64) -> f64 {
    if total <= 0.0 {
        0.0
    } else {
        (used / total) * 100.0
    }
}

fn draw_cpu_gauge(f: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let cpu_usage = app.system.global_cpu_info().cpu_usage();
    let color = theme.status_color(cpu_usage, 50.0, 80.0);

    let gauge = LineGauge::default()
        .block(
            Block::default()
                .title(Span::styled(" CPU ", Style::default().fg(theme.primary)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border_style()),
        )
        .gauge_style(Style::default().fg(color))
        .line_set(symbols::line::THICK)
        .ratio(gauge_ratio(cpu_usage as f64))
        .label(format!("{:.1}%", cpu_usage));

    f.render_widget(gauge, area);
}

fn draw_memory_gauge(f: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let used = app.system.used_memory() as f64;
    let total = app.system.total_memory() as f64;
    let percentage = used_percentage(used, total);
    let color = theme.status_color(percentage as f32, 60.0, 85.0);

    let used_gb = used / 1024.0 / 1024.0 / 1024.0;
    let total_gb = total / 1024.0 / 1024.0 / 1024.0;

    let gauge = LineGauge::default()
        .block(
            Block::default()
                .title(Span::styled(" Memory ", Style::default().fg(theme.primary)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border_style()),
        )
        .gauge_style(Style::default().fg(color))
        .line_set(symbols::line::THICK)
        .ratio(gauge_ratio(percentage))
        .label(format!(
            "{:.1}GB / {:.1}GB ({:.0}%)",
            used_gb, total_gb, percentage
        ));

    f.render_widget(gauge, area);
}

fn draw_cpu_sparkline(f: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let data: Vec<u64> = app.cpu_history.iter().map(|&v| v as u64).collect();

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(" CPU History (60s) ")
                .title_style(Style::default().fg(theme.muted))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border_style()),
        )
        .data(&data)
        .max(100)
        .style(Style::default().fg(theme.info));

    f.render_widget(sparkline, area);
}

fn draw_memory_sparkline(f: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let data: Vec<u64> = app.memory_history.iter().map(|&v| v as u64).collect();

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(" Memory History (60s) ")
                .title_style(Style::default().fg(theme.muted))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border_style()),
        )
        .data(&data)
        .max(100)
        .style(Style::default().fg(theme.secondary));

    f.render_widget(sparkline, area);
}

fn draw_search_bar(f: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let search_style = if matches!(app.mode, AppMode::Search) {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.muted)
    };

    let cursor_suffix = if matches!(app.mode, AppMode::Search) {
        "_"
    } else {
        ""
    };

    let search = Paragraph::new(format!("Search: {}{}", app.search_query, cursor_suffix))
        .style(search_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(if matches!(app.mode, AppMode::Search) {
                    theme.border_focused_style()
                } else {
                    theme.border_style()
                })
                .title(" Filter "),
        );

    f.render_widget(search, area);
}

fn draw_process_table(f: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    // Build header with sort indicators
    let header_cells = [
        ("", SortColumn::Pid),
        ("PID", SortColumn::Pid),
        ("Name", SortColumn::Name),
        ("CPU%", SortColumn::Cpu),
        ("Memory", SortColumn::Memory),
        ("Status", SortColumn::Status),
        ("Uptime", SortColumn::Status),
    ]
    .iter()
    .map(|(name, col)| {
        let text = if *col == app.sort_column && !name.is_empty() {
            let arrow = match app.sort_order {
                SortOrder::Ascending => "^",
                SortOrder::Descending => "v",
            };
            format!("{} {}", name, arrow)
        } else {
            name.to_string()
        };
        Cell::from(text).style(theme.header_style())
    });

    let header = Row::new(header_cells).height(1).bottom_margin(1);

    // Create rows from cached process list
    let rows: Vec<Row> = app
        .process_list
        .iter()
        .map(|proc| {
            let indicator = theme.status_indicator(&proc.status);
            let cpu_color = theme.status_color(proc.cpu_usage, 30.0, 70.0);

            let cells = vec![
                Cell::from(indicator).style(theme.process_status_style(&proc.status)),
                Cell::from(format!("{}", proc.pid)),
                Cell::from(proc.name.clone()),
                Cell::from(format!("{:.1}%", proc.cpu_usage)).style(Style::default().fg(cpu_color)),
                Cell::from(format!("{} MB", proc.memory_mb)),
                Cell::from(proc.status.clone()).style(theme.process_status_style(&proc.status)),
                Cell::from(format_uptime(proc.run_time)).style(Style::default().fg(theme.muted)),
            ];

            Row::new(cells).height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(2),      // Status indicator
        Constraint::Length(8),      // PID
        Constraint::Percentage(35), // Name
        Constraint::Length(8),      // CPU%
        Constraint::Length(12),     // Memory
        Constraint::Length(10),     // Status
        Constraint::Length(12),     // Uptime
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(" Monitored Services ", Style::default().fg(theme.primary)),
                    Span::styled(
                        format!("({} processes)", app.process_list.len()),
                        Style::default().fg(theme.muted),
                    ),
                ]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border_style()),
        )
        .column_spacing(1)
        .highlight_style(theme.selected_style())
        .highlight_symbol("▶ ");

    f.render_stateful_widget(table, area, &mut app.table_state);

    // Add scrollbar if many processes
    let visible_rows = area.height.saturating_sub(4) as usize;
    if app.process_list.len() > visible_rows {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));
        let mut scrollbar_state = ScrollbarState::new(app.process_list.len())
            .position(app.table_state.selected().unwrap_or(0));
        f.render_stateful_widget(
            scrollbar,
            area.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

fn draw_process_table_minimal(f: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let header_cells = ["PID", "Name", "CPU%"]
        .iter()
        .map(|h| Cell::from(*h).style(theme.header_style()));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .process_list
        .iter()
        .map(|proc| {
            Row::new(vec![
                Cell::from(format!("{}", proc.pid)),
                Cell::from(proc.name.clone()),
                Cell::from(format!("{:.1}%", proc.cpu_usage)),
            ])
            .height(1)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Percentage(60),
            Constraint::Length(6),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL))
    .highlight_style(theme.selected_style());

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_footer(f: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let mode_str = match app.mode {
        AppMode::Normal => "NORMAL",
        AppMode::Search => "SEARCH",
        AppMode::Help => "HELP",
        AppMode::Confirm(_) => "CONFIRM",
    };

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(theme.border_style())
        .border_type(BorderType::Rounded);

    let help_text = Line::from(vec![
        Span::styled(
            format!(" [{}] ", mode_str),
            Style::default().fg(theme.accent),
        ),
        Span::styled(&app.config_path, Style::default().fg(theme.muted)),
        Span::styled(" | ", Style::default().fg(theme.border)),
        Span::styled("[q]", Style::default().fg(theme.primary)),
        Span::styled("uit ", Style::default().fg(theme.muted)),
        Span::styled("[?]", Style::default().fg(theme.primary)),
        Span::styled("help ", Style::default().fg(theme.muted)),
        Span::styled("[/]", Style::default().fg(theme.primary)),
        Span::styled("search ", Style::default().fg(theme.muted)),
        Span::styled("[s]", Style::default().fg(theme.primary)),
        Span::styled("ort ", Style::default().fg(theme.muted)),
    ]);

    let footer = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}

fn draw_footer_compact(f: &mut Frame, _app: &App, theme: &Theme, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("[q]uit ", Style::default().fg(theme.muted)),
        Span::styled("[r]efresh ", Style::default().fg(theme.muted)),
        Span::styled("[j/k]nav ", Style::default().fg(theme.muted)),
        Span::styled("[?]help", Style::default().fg(theme.muted)),
    ]))
    .alignment(Alignment::Center);

    f.render_widget(footer, area);
}

fn draw_help_overlay(f: &mut Frame, theme: &Theme) {
    let area = centered_rect(60, 80, f.size());

    f.render_widget(Clear, area);

    let help_text = vec![
        "",
        "  NAVIGATION",
        "  ----------",
        "  j/Down      Move selection down",
        "  k/Up        Move selection up",
        "  g/Home      Jump to first item",
        "  G/End       Jump to last item",
        "  Ctrl+d/PgDn Page down",
        "  Ctrl+u/PgUp Page up",
        "",
        "  SORTING",
        "  -------",
        "  s           Cycle sort column",
        "  S           Toggle sort order",
        "  1-5         Sort by column",
        "",
        "  FILTERING",
        "  ---------",
        "  /           Enter search mode",
        "  Esc         Clear search / Exit",
        "",
        "  ACTIONS",
        "  -------",
        "  x/Del       Kill selected process",
        "  R           Restart selected service",
        "  r           Refresh process list",
        "",
        "  Press any key to close",
    ];

    let help_paragraph = Paragraph::new(help_text.join("\n"))
        .block(
            Block::default()
                .title(" Keyboard Shortcuts ")
                .title_style(theme.title_style())
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(theme.border_focused_style()),
        )
        .style(Style::default().fg(theme.fg))
        .alignment(Alignment::Left);

    f.render_widget(help_paragraph, area);
}

fn draw_confirm_dialog(f: &mut Frame, theme: &Theme, action: ConfirmAction) {
    let area = centered_rect(50, 25, f.size());

    f.render_widget(Clear, area);

    let (title, message) = match action {
        ConfirmAction::KillProcess(pid) => (
            " Confirm Kill ",
            format!(
                "Are you sure you want to kill process {}?\n\n[Y]es / [N]o",
                pid
            ),
        ),
        ConfirmAction::RestartService(pid) => (
            " Confirm Restart ",
            format!(
                "Are you sure you want to restart service (PID {})?\n\n[Y]es / [N]o",
                pid
            ),
        ),
    };

    let dialog = Paragraph::new(message)
        .block(
            Block::default()
                .title(title)
                .title_style(theme.title_style())
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(theme.border_focused_style()),
        )
        .style(Style::default().fg(theme.accent))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(dialog, area);
}

/// Create a centered rectangle with percentage dimensions
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let [area] = Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .areas(area);
    area
}

/// Format process uptime to human readable string
fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h{}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m{}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::app::ProcessInfo;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn sample_app() -> App {
        App::with_processes(vec![
            ProcessInfo {
                pid: 4242,
                name: "rustywatch-api".to_string(),
                cpu_usage: 12.5,
                memory_mb: 128,
                status: "Run".to_string(),
                run_time: 3671,
            },
            ProcessInfo {
                pid: 4243,
                name: "rustywatch-web".to_string(),
                cpu_usage: 3.5,
                memory_mb: 64,
                status: "Sleep".to_string(),
                run_time: 12,
            },
        ])
    }

    /// Renders `app` into an off-screen terminal and returns the flattened text.
    fn render(app: &mut App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();

        let buffer = terminal.backend().buffer().clone();
        buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<Vec<_>>()
            .join("")
    }

    #[test]
    fn test_full_layout_renders_processes() {
        let mut app = sample_app();
        let out = render(&mut app, 140, 40);

        assert!(out.contains("RustyWatch"));
        assert!(out.contains("rustywatch-api"));
        assert!(out.contains("4242"));
        assert!(out.contains("NORMAL"), "footer shows the current mode");
    }

    #[test]
    fn test_compact_layout_renders() {
        let mut app = sample_app();
        let out = render(&mut app, 100, 30);

        assert!(out.contains("RustyWatch"));
        assert!(out.contains("rustywatch-api"));
    }

    #[test]
    fn test_minimal_layout_renders() {
        let mut app = sample_app();
        let out = render(&mut app, 60, 20);

        assert!(out.contains("RustyWatch"));
        assert!(out.contains("[q]uit"));
    }

    #[test]
    fn test_empty_process_list_shows_placeholder() {
        let mut app = App::with_processes(Vec::new());
        let out = render(&mut app, 140, 40);

        assert!(out.contains("RustyWatch"));
    }

    #[test]
    fn test_search_bar_shows_query() {
        let mut app = sample_app();
        app.mode = AppMode::Search;
        app.search_query = "api".to_string();

        let out = render(&mut app, 140, 40);
        assert!(out.contains("api"));
        assert!(out.contains("SEARCH"));
    }

    // A non-empty query must keep the search bar visible after leaving search
    // mode, otherwise the filter is applied invisibly.
    #[test]
    fn test_search_bar_visible_with_query_in_normal_mode() {
        let mut app = sample_app();
        app.search_query = "rustywatch".to_string();

        let out = render(&mut app, 100, 30);
        assert!(out.contains("rustywatch"));
    }

    #[test]
    fn test_help_overlay_renders() {
        let mut app = sample_app();
        app.mode = AppMode::Help;

        let out = render(&mut app, 140, 40);
        assert!(out.contains("Keyboard Shortcuts"));
        assert!(out.contains("NAVIGATION"));
    }

    #[test]
    fn test_confirm_kill_dialog_names_the_pid() {
        let mut app = sample_app();
        app.mode = AppMode::Confirm(ConfirmAction::KillProcess(4242));

        let out = render(&mut app, 140, 40);
        assert!(out.contains("Confirm Kill"));
        assert!(out.contains("4242"));
    }

    #[test]
    fn test_confirm_restart_dialog_renders() {
        let mut app = sample_app();
        app.mode = AppMode::Confirm(ConfirmAction::RestartService(4243));

        let out = render(&mut app, 140, 40);
        assert!(out.contains("Confirm Restart"));
    }

    // Every layout branch must survive a terminal too small to hold it.
    #[test]
    fn test_tiny_terminal_does_not_panic() {
        for (w, h) in [(20, 5), (40, 10), (80, 12), (120, 30)] {
            let mut app = sample_app();
            app.mode = AppMode::Help;
            render(&mut app, w, h);
        }
    }

    // `LineGauge::ratio` panics outside 0.0..=1.0, and an unreadable memory
    // total yields NaN — that combination used to take down the whole TUI.
    #[test]
    fn test_gauge_ratio_normalizes_out_of_range_and_nan() {
        assert_eq!(gauge_ratio(f64::NAN), 0.0);
        assert_eq!(gauge_ratio(-10.0), 0.0);
        assert_eq!(gauge_ratio(0.0), 0.0);
        assert_eq!(gauge_ratio(50.0), 0.5);
        assert_eq!(gauge_ratio(100.0), 1.0);
        assert_eq!(gauge_ratio(400.0), 1.0);
        assert_eq!(gauge_ratio(f64::INFINITY), 1.0);
    }

    #[test]
    fn test_used_percentage_guards_zero_total() {
        assert_eq!(used_percentage(100.0, 0.0), 0.0);
        assert_eq!(used_percentage(0.0, 0.0), 0.0);
        assert_eq!(used_percentage(25.0, 100.0), 25.0);
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(0), "0s");
        assert_eq!(format_uptime(45), "45s");
        assert_eq!(format_uptime(60), "1m0s");
        assert_eq!(format_uptime(125), "2m5s");
        assert_eq!(format_uptime(3600), "1h0m");
        assert_eq!(format_uptime(3671), "1h1m");
        assert_eq!(format_uptime(90_061), "25h1m");
    }

    #[test]
    fn test_centered_rect_is_centered_and_scaled() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(50, 20, area);

        assert_eq!(centered.width, 50);
        assert_eq!(centered.height, 20);
        assert_eq!(centered.x, 25);
        assert_eq!(centered.y, 40);
    }

    #[test]
    fn test_centered_rect_full_size() {
        let area = Rect::new(0, 0, 80, 24);
        assert_eq!(centered_rect(100, 100, area), area);
    }
}
