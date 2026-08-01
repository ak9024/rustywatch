use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::TableState, Terminal};
use std::{
    collections::VecDeque,
    error::Error,
    io,
    time::{Duration, Instant},
};
use sysinfo::{Pid, Signal, System};

use crate::monitor::service::ServiceTracker;
use crate::monitor::ui;

/// Column to sort process table by
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Pid,
    Name,
    Cpu,
    Memory,
    Status,
}

/// Sort order for process table
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Current mode of the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Search,
    Help,
    Confirm(ConfirmAction),
}

/// Action requiring confirmation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    KillProcess(u32),
    RestartService(u32),
}

/// Cached process information for display
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: u64,
    pub status: String,
    pub run_time: u64,
}

/// Returns `true` when `process` matches `query` by name or PID.
///
/// An empty query matches everything. Matching is case-insensitive.
pub fn matches_search(process: &ProcessInfo, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let query = query.to_lowercase();
    process.name.to_lowercase().contains(&query) || process.pid.to_string().contains(&query)
}

/// Sorts `processes` in place by `column`, honouring `order`.
pub fn sort_processes(processes: &mut [ProcessInfo], column: SortColumn, order: SortOrder) {
    processes.sort_by(|a, b| {
        let cmp = match column {
            SortColumn::Pid => a.pid.cmp(&b.pid),
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Cpu => a
                .cpu_usage
                .partial_cmp(&b.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::Memory => a.memory_mb.cmp(&b.memory_mb),
            SortColumn::Status => a.status.cmp(&b.status),
        };

        match order {
            SortOrder::Ascending => cmp,
            SortOrder::Descending => cmp.reverse(),
        }
    });
}

pub struct App {
    pub system: System,
    pub service_tracker: ServiceTracker,
    pub should_quit: bool,
    pub refresh_interval: Duration,
    pub last_refresh: Instant,
    pub config_path: String,

    // Table interaction
    pub table_state: TableState,
    pub process_list: Vec<ProcessInfo>,

    // Sorting
    pub sort_column: SortColumn,
    pub sort_order: SortOrder,

    // Filtering
    pub search_query: String,
    pub mode: AppMode,

    // History for graphs (60 data points = 60 seconds at 1s refresh)
    pub cpu_history: VecDeque<f32>,
    pub memory_history: VecDeque<f64>,
    history_max_len: usize,
}

impl App {
    pub fn new(config_path: String) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let mut service_tracker = ServiceTracker::new();
        if let Err(e) = service_tracker.load_services(&config_path) {
            eprintln!("Error loading services: {}", e);
        }
        // Initial discovery using sysinfo
        service_tracker.discover_service_processes(&system);

        let mut table_state = TableState::default();
        table_state.select(Some(0));

        let mut app = App {
            system,
            service_tracker,
            should_quit: false,
            refresh_interval: Duration::from_secs(1),
            last_refresh: Instant::now(),
            config_path,
            table_state,
            process_list: Vec::new(),
            sort_column: SortColumn::Cpu,
            sort_order: SortOrder::Descending,
            search_query: String::new(),
            mode: AppMode::Normal,
            cpu_history: VecDeque::with_capacity(60),
            memory_history: VecDeque::with_capacity(60),
            history_max_len: 60,
        };

        app.update_process_list();
        app.update_history();
        app
    }

    /// Refresh process data
    pub fn refresh(&mut self) {
        self.system.refresh_processes();
        self.service_tracker.smart_refresh(&self.system);
        self.update_process_list();
        self.update_history();
        self.last_refresh = Instant::now();
    }

    /// Update CPU and memory history for sparklines
    fn update_history(&mut self) {
        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        if self.cpu_history.len() >= self.history_max_len {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(cpu_usage);

        // A zero total (unreadable on some platforms) would push NaN into the
        // history and poison the sparkline.
        let total = self.system.total_memory() as f64;
        let mem_percent = if total <= 0.0 {
            0.0
        } else {
            (self.system.used_memory() as f64 / total) * 100.0
        };
        if self.memory_history.len() >= self.history_max_len {
            self.memory_history.pop_front();
        }
        self.memory_history.push_back(mem_percent);
    }

    /// Update cached process list from system
    pub fn update_process_list(&mut self) {
        self.process_list = self
            .system
            .processes()
            .iter()
            .filter(|(pid, _)| self.service_tracker.is_service_process(&pid.as_u32()))
            .map(|(pid, proc)| ProcessInfo {
                pid: pid.as_u32(),
                name: proc.name().to_string(),
                cpu_usage: proc.cpu_usage(),
                memory_mb: proc.memory() / 1024 / 1024,
                status: format!("{:?}", proc.status()),
                run_time: proc.run_time(),
            })
            .filter(|p| self.matches_search(p))
            .collect();

        self.sort_processes();

        // Ensure selection is valid
        if let Some(selected) = self.table_state.selected() {
            if selected >= self.process_list.len() && !self.process_list.is_empty() {
                self.table_state.select(Some(self.process_list.len() - 1));
            }
        }
    }

    /// Check if process matches current search query
    fn matches_search(&self, process: &ProcessInfo) -> bool {
        matches_search(process, &self.search_query)
    }

    /// Sort process list by current column and order
    fn sort_processes(&mut self) {
        sort_processes(&mut self.process_list, self.sort_column, self.sort_order);
    }

    // Navigation methods

    /// Move selection down
    pub fn next(&mut self) {
        if self.process_list.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.process_list.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Move selection up
    pub fn previous(&mut self) {
        if self.process_list.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.process_list.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Jump to first item
    pub fn first(&mut self) {
        if !self.process_list.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    /// Jump to last item
    pub fn last(&mut self) {
        if !self.process_list.is_empty() {
            self.table_state.select(Some(self.process_list.len() - 1));
        }
    }

    /// Page down
    pub fn page_down(&mut self, rows: usize) {
        if self.process_list.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => std::cmp::min(i + rows, self.process_list.len() - 1),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Page up
    pub fn page_up(&mut self, rows: usize) {
        if self.process_list.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => i.saturating_sub(rows),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    // Sorting methods

    /// Set sort column (toggles order if same column)
    pub fn set_sort_column(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.toggle_sort_order();
        } else {
            self.sort_column = column;
            self.sort_order = SortOrder::Descending;
        }
        self.sort_processes();
    }

    /// Cycle to next sort column
    pub fn next_sort_column(&mut self) {
        self.sort_column = match self.sort_column {
            SortColumn::Pid => SortColumn::Name,
            SortColumn::Name => SortColumn::Cpu,
            SortColumn::Cpu => SortColumn::Memory,
            SortColumn::Memory => SortColumn::Status,
            SortColumn::Status => SortColumn::Pid,
        };
        self.sort_processes();
    }

    /// Toggle sort order
    pub fn toggle_sort_order(&mut self) {
        self.sort_order = match self.sort_order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
        self.sort_processes();
    }

    // Search methods

    /// Enter search mode
    pub fn enter_search_mode(&mut self) {
        self.mode = AppMode::Search;
    }

    /// Exit search mode
    pub fn exit_search_mode(&mut self) {
        self.mode = AppMode::Normal;
    }

    /// Add character to search query
    pub fn search_push(&mut self, c: char) {
        self.search_query.push(c);
        self.update_process_list();
    }

    /// Remove last character from search query
    pub fn search_pop(&mut self) {
        self.search_query.pop();
        self.update_process_list();
    }

    /// Clear search query
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.update_process_list();
    }

    // Process action methods

    /// Get currently selected PID
    pub fn selected_pid(&self) -> Option<u32> {
        self.table_state
            .selected()
            .and_then(|i| self.process_list.get(i))
            .map(|p| p.pid)
    }

    /// Request to kill selected process
    pub fn request_kill_process(&mut self) {
        if let Some(pid) = self.selected_pid() {
            self.mode = AppMode::Confirm(ConfirmAction::KillProcess(pid));
        }
    }

    /// Request to restart selected service
    pub fn request_restart_service(&mut self) {
        if let Some(pid) = self.selected_pid() {
            self.mode = AppMode::Confirm(ConfirmAction::RestartService(pid));
        }
    }

    /// Confirm pending action
    pub fn confirm_action(&mut self) {
        if let AppMode::Confirm(action) = self.mode {
            match action {
                ConfirmAction::KillProcess(pid) | ConfirmAction::RestartService(pid) => {
                    self.kill_process(pid);
                }
            }
        }
        self.mode = AppMode::Normal;
    }

    /// Cancel pending action
    pub fn cancel_action(&mut self) {
        self.mode = AppMode::Normal;
    }

    /// Kill a process by PID
    fn kill_process(&mut self, pid: u32) {
        if let Some(process) = self.system.process(Pid::from_u32(pid)) {
            process.kill_with(Signal::Term);
            // Refresh after kill
            self.refresh();
        }
    }

    // Key handling

    pub fn on_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match self.mode {
            AppMode::Normal => self.handle_normal_mode(key, modifiers),
            AppMode::Search => self.handle_search_mode(key),
            AppMode::Help => self.handle_help_mode(key),
            AppMode::Confirm(_) => self.handle_confirm_mode(key),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            // Quit
            KeyCode::Char('q') => self.should_quit = true,

            // Refresh
            KeyCode::Char('r') => self.refresh(),

            // Navigation
            KeyCode::Down | KeyCode::Char('j') => self.next(),
            KeyCode::Up | KeyCode::Char('k') => self.previous(),
            KeyCode::Home | KeyCode::Char('g') => self.first(),
            KeyCode::End | KeyCode::Char('G') => self.last(),
            KeyCode::PageDown => self.page_down(10),
            KeyCode::PageUp => self.page_up(10),
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => self.page_down(10),
            KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => self.page_up(10),

            // Sorting
            KeyCode::Char('s') => self.next_sort_column(),
            KeyCode::Char('S') => self.toggle_sort_order(),
            KeyCode::Char('1') => self.set_sort_column(SortColumn::Pid),
            KeyCode::Char('2') => self.set_sort_column(SortColumn::Name),
            KeyCode::Char('3') => self.set_sort_column(SortColumn::Cpu),
            KeyCode::Char('4') => self.set_sort_column(SortColumn::Memory),
            KeyCode::Char('5') => self.set_sort_column(SortColumn::Status),

            // Search
            KeyCode::Char('/') => self.enter_search_mode(),
            KeyCode::Esc => self.clear_search(),

            // Process actions
            KeyCode::Char('x') | KeyCode::Delete => self.request_kill_process(),
            KeyCode::Char('R') => self.request_restart_service(),

            // Help
            KeyCode::Char('?') | KeyCode::F(1) => self.mode = AppMode::Help,

            _ => {}
        }
    }

    fn handle_search_mode(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Enter => self.exit_search_mode(),
            KeyCode::Backspace => self.search_pop(),
            KeyCode::Char(c) => self.search_push(c),
            _ => {}
        }
    }

    fn handle_help_mode(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::F(1) => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_confirm_mode(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => self.confirm_action(),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => self.cancel_action(),
            _ => {}
        }
    }
}

#[cfg(test)]
impl App {
    /// Builds an `App` around a fixed process list without touching the real
    /// system, so tests are deterministic and cannot signal live processes.
    ///
    /// `system` is left empty: `kill_process` looks the PID up there and finds
    /// nothing, and `update_process_list` would replace `process_list` — tests
    /// on the injected list must not call `refresh`.
    pub(crate) fn with_processes(process_list: Vec<ProcessInfo>) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        App {
            system: System::new(),
            service_tracker: ServiceTracker::new(),
            should_quit: false,
            refresh_interval: Duration::from_secs(1),
            last_refresh: Instant::now(),
            config_path: "rustywatch.yaml".to_string(),
            table_state,
            process_list,
            sort_column: SortColumn::Cpu,
            sort_order: SortOrder::Descending,
            search_query: String::new(),
            mode: AppMode::Normal,
            cpu_history: VecDeque::from(vec![10.0, 20.0, 30.0]),
            memory_history: VecDeque::from(vec![40.0, 50.0, 60.0]),
            history_max_len: 60,
        }
    }
}

pub fn run(config_path: String) -> Result<(), Box<dyn Error>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(config_path);

    // Main loop
    loop {
        // Draw UI (pass mutable ref for TableState)
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    app.on_key(key.code, key.modifiers);
                }
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }

        // Auto-refresh based on interval
        if app.last_refresh.elapsed() >= app.refresh_interval {
            app.refresh();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_config() -> NamedTempFile {
        let config_content = r#"
workspaces:
  - dir: "."
    cmd: "echo test"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file
    }

    #[test]
    fn test_app_new() {
        let temp_file = create_test_config();
        let config_path = temp_file.path().to_str().unwrap().to_string();

        let app = App::new(config_path.clone());

        assert!(!app.should_quit);
        assert_eq!(app.config_path, config_path);
        assert_eq!(app.refresh_interval, Duration::from_secs(1));
        assert_eq!(app.mode, AppMode::Normal);
        assert_eq!(app.sort_column, SortColumn::Cpu);
        assert_eq!(app.sort_order, SortOrder::Descending);
    }

    #[test]
    fn test_on_key_quit() {
        let temp_file = create_test_config();
        let config_path = temp_file.path().to_str().unwrap().to_string();

        let mut app = App::new(config_path);
        assert!(!app.should_quit);

        app.on_key(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(app.should_quit);
    }

    #[test]
    fn test_on_key_refresh() {
        let temp_file = create_test_config();
        let config_path = temp_file.path().to_str().unwrap().to_string();

        let mut app = App::new(config_path);
        let initial_refresh = app.last_refresh;

        // Wait a bit to ensure time difference
        std::thread::sleep(Duration::from_millis(10));

        app.on_key(KeyCode::Char('r'), KeyModifiers::NONE);

        // last_refresh should be updated
        assert!(app.last_refresh > initial_refresh);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_navigation() {
        let temp_file = create_test_config();
        let config_path = temp_file.path().to_str().unwrap().to_string();

        let mut app = App::new(config_path);

        // Test next/previous with empty list
        app.next();
        app.previous();
        assert_eq!(app.table_state.selected(), Some(0));
    }

    #[test]
    fn test_sorting() {
        let temp_file = create_test_config();
        let config_path = temp_file.path().to_str().unwrap().to_string();

        let mut app = App::new(config_path);

        // Default sort
        assert_eq!(app.sort_column, SortColumn::Cpu);
        assert_eq!(app.sort_order, SortOrder::Descending);

        // Cycle sort column
        app.next_sort_column();
        assert_eq!(app.sort_column, SortColumn::Memory);

        // Toggle order
        app.toggle_sort_order();
        assert_eq!(app.sort_order, SortOrder::Ascending);

        // Set specific column
        app.set_sort_column(SortColumn::Name);
        assert_eq!(app.sort_column, SortColumn::Name);
        assert_eq!(app.sort_order, SortOrder::Descending);
    }

    #[test]
    fn test_search_mode() {
        let temp_file = create_test_config();
        let config_path = temp_file.path().to_str().unwrap().to_string();

        let mut app = App::new(config_path);

        // Enter search mode
        app.on_key(KeyCode::Char('/'), KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Search);

        // Type search query
        app.on_key(KeyCode::Char('t'), KeyModifiers::NONE);
        app.on_key(KeyCode::Char('e'), KeyModifiers::NONE);
        app.on_key(KeyCode::Char('s'), KeyModifiers::NONE);
        app.on_key(KeyCode::Char('t'), KeyModifiers::NONE);
        assert_eq!(app.search_query, "test");

        // Backspace
        app.on_key(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(app.search_query, "tes");

        // Exit search mode
        app.on_key(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn test_help_mode() {
        let temp_file = create_test_config();
        let config_path = temp_file.path().to_str().unwrap().to_string();

        let mut app = App::new(config_path);

        // Enter help mode
        app.on_key(KeyCode::Char('?'), KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Help);

        // Exit help mode
        app.on_key(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn test_app_with_nonexistent_config() {
        // App should handle missing config gracefully
        let app = App::new("/nonexistent/config.yaml".to_string());

        assert!(!app.should_quit);
        assert!(app.service_tracker.service_commands.is_empty());
    }

    // --- deterministic tests over an injected process list ---

    fn process(pid: u32, name: &str, cpu: f32, memory_mb: u64, status: &str) -> ProcessInfo {
        ProcessInfo {
            pid,
            name: name.to_string(),
            cpu_usage: cpu,
            memory_mb,
            status: status.to_string(),
            run_time: 42,
        }
    }

    fn sample_processes() -> Vec<ProcessInfo> {
        vec![
            process(300, "beta", 5.0, 900, "Run"),
            process(100, "Alpha", 50.0, 100, "Sleep"),
            process(200, "gamma", 20.0, 500, "Stop"),
        ]
    }

    fn pids(app: &App) -> Vec<u32> {
        app.process_list.iter().map(|p| p.pid).collect()
    }

    #[test]
    fn test_sort_processes_by_every_column() {
        let mut list = sample_processes();

        sort_processes(&mut list, SortColumn::Pid, SortOrder::Ascending);
        assert_eq!(
            list.iter().map(|p| p.pid).collect::<Vec<_>>(),
            vec![100, 200, 300]
        );

        sort_processes(&mut list, SortColumn::Pid, SortOrder::Descending);
        assert_eq!(
            list.iter().map(|p| p.pid).collect::<Vec<_>>(),
            vec![300, 200, 100]
        );

        // Name sorting is case-insensitive: "Alpha" must lead, not trail.
        sort_processes(&mut list, SortColumn::Name, SortOrder::Ascending);
        assert_eq!(
            list.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
            vec!["Alpha", "beta", "gamma"]
        );

        sort_processes(&mut list, SortColumn::Cpu, SortOrder::Descending);
        assert_eq!(
            list.iter().map(|p| p.pid).collect::<Vec<_>>(),
            vec![100, 200, 300]
        );

        sort_processes(&mut list, SortColumn::Memory, SortOrder::Ascending);
        assert_eq!(
            list.iter().map(|p| p.memory_mb).collect::<Vec<_>>(),
            vec![100, 500, 900]
        );

        sort_processes(&mut list, SortColumn::Status, SortOrder::Ascending);
        assert_eq!(
            list.iter().map(|p| p.status.as_str()).collect::<Vec<_>>(),
            vec!["Run", "Sleep", "Stop"]
        );
    }

    #[test]
    fn test_sort_processes_handles_nan_cpu() {
        let mut list = vec![
            process(1, "a", f32::NAN, 1, "Run"),
            process(2, "b", 1.0, 1, "Run"),
        ];

        // Must not panic: `partial_cmp` returns `None` for NaN.
        sort_processes(&mut list, SortColumn::Cpu, SortOrder::Descending);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_matches_search() {
        let p = process(1234, "rustywatch", 1.0, 1, "Run");

        assert!(matches_search(&p, ""));
        assert!(matches_search(&p, "rusty"));
        assert!(matches_search(&p, "RUSTY"), "search is case-insensitive");
        assert!(matches_search(&p, "234"), "matches on PID substring");
        assert!(!matches_search(&p, "nope"));
    }

    #[test]
    fn test_set_sort_column_reorders_and_toggles() {
        let mut app = App::with_processes(sample_processes());

        app.set_sort_column(SortColumn::Pid);
        assert_eq!(app.sort_order, SortOrder::Descending);
        assert_eq!(pids(&app), vec![300, 200, 100]);

        // Same column again flips the order rather than re-selecting it.
        app.set_sort_column(SortColumn::Pid);
        assert_eq!(app.sort_order, SortOrder::Ascending);
        assert_eq!(pids(&app), vec![100, 200, 300]);
    }

    #[test]
    fn test_next_sort_column_cycles_back_to_pid() {
        let mut app = App::with_processes(sample_processes());
        app.sort_column = SortColumn::Pid;

        for expected in [
            SortColumn::Name,
            SortColumn::Cpu,
            SortColumn::Memory,
            SortColumn::Status,
            SortColumn::Pid,
        ] {
            app.next_sort_column();
            assert_eq!(app.sort_column, expected);
        }
    }

    #[test]
    fn test_navigation_wraps_around() {
        let mut app = App::with_processes(sample_processes());

        assert_eq!(app.table_state.selected(), Some(0));
        app.next();
        app.next();
        assert_eq!(app.table_state.selected(), Some(2));

        // Past the end wraps to the start, and back again from the start.
        app.next();
        assert_eq!(app.table_state.selected(), Some(0));
        app.previous();
        assert_eq!(app.table_state.selected(), Some(2));
    }

    #[test]
    fn test_first_last_and_paging() {
        let mut app = App::with_processes(sample_processes());

        app.last();
        assert_eq!(app.table_state.selected(), Some(2));
        app.first();
        assert_eq!(app.table_state.selected(), Some(0));

        // Paging clamps instead of running off either end.
        app.page_down(10);
        assert_eq!(app.table_state.selected(), Some(2));
        app.page_up(10);
        assert_eq!(app.table_state.selected(), Some(0));
    }

    #[test]
    fn test_navigation_on_empty_list_is_a_no_op() {
        let mut app = App::with_processes(Vec::new());
        app.table_state.select(None);

        app.next();
        app.previous();
        app.first();
        app.last();
        app.page_down(5);
        app.page_up(5);

        assert_eq!(app.table_state.selected(), None);
    }

    #[test]
    fn test_selected_pid_tracks_selection() {
        let mut app = App::with_processes(sample_processes());
        app.set_sort_column(SortColumn::Pid);

        assert_eq!(app.selected_pid(), Some(300));
        app.next();
        assert_eq!(app.selected_pid(), Some(200));

        app.table_state.select(None);
        assert_eq!(app.selected_pid(), None);
    }

    #[test]
    fn test_selected_pid_none_when_list_empty() {
        assert_eq!(App::with_processes(Vec::new()).selected_pid(), None);
    }

    // The injected `System` is empty, so confirming never reaches a live
    // process — this exercises the mode transitions only.
    #[test]
    fn test_kill_request_confirm_and_cancel() {
        let mut app = App::with_processes(sample_processes());
        app.set_sort_column(SortColumn::Pid);

        app.on_key(KeyCode::Char('x'), KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Confirm(ConfirmAction::KillProcess(300)));

        app.on_key(KeyCode::Char('n'), KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Normal);

        app.on_key(KeyCode::Delete, KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Confirm(ConfirmAction::KillProcess(300)));

        app.on_key(KeyCode::Char('y'), KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn test_restart_request_uses_its_own_action() {
        let mut app = App::with_processes(sample_processes());
        app.set_sort_column(SortColumn::Pid);

        app.on_key(KeyCode::Char('R'), KeyModifiers::NONE);
        assert_eq!(
            app.mode,
            AppMode::Confirm(ConfirmAction::RestartService(300))
        );

        app.on_key(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn test_kill_request_ignored_without_a_selection() {
        let mut app = App::with_processes(Vec::new());

        app.on_key(KeyCode::Char('x'), KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn test_confirm_mode_ignores_unrelated_keys() {
        let mut app = App::with_processes(sample_processes());
        app.mode = AppMode::Confirm(ConfirmAction::KillProcess(1));

        app.on_key(KeyCode::Char('z'), KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Confirm(ConfirmAction::KillProcess(1)));
    }

    #[test]
    fn test_help_mode_swallows_navigation() {
        let mut app = App::with_processes(sample_processes());

        app.on_key(KeyCode::F(1), KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Help);

        // While the overlay is up, `j` must not move the selection.
        app.on_key(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(app.table_state.selected(), Some(0));

        app.on_key(KeyCode::Char('?'), KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn test_help_mode_quit_key_closes_overlay_without_quitting() {
        let mut app = App::with_processes(sample_processes());
        app.mode = AppMode::Help;

        app.on_key(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(app.mode, AppMode::Normal);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_search_mode_enter_confirms() {
        let mut app = App::with_processes(Vec::new());

        app.on_key(KeyCode::Char('/'), KeyModifiers::NONE);
        app.on_key(KeyCode::Char('a'), KeyModifiers::NONE);
        app.on_key(KeyCode::Enter, KeyModifiers::NONE);

        assert_eq!(app.mode, AppMode::Normal);
        assert_eq!(app.search_query, "a");
    }

    #[test]
    fn test_search_mode_ignores_unhandled_keys() {
        let mut app = App::with_processes(Vec::new());
        app.mode = AppMode::Search;

        app.on_key(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(app.search_query, "");
        assert_eq!(app.mode, AppMode::Search);
    }

    #[test]
    fn test_ctrl_d_and_ctrl_u_page() {
        let mut app = App::with_processes(sample_processes());

        app.on_key(KeyCode::Char('d'), KeyModifiers::CONTROL);
        assert_eq!(app.table_state.selected(), Some(2));

        app.on_key(KeyCode::Char('u'), KeyModifiers::CONTROL);
        assert_eq!(app.table_state.selected(), Some(0));
    }

    #[test]
    fn test_number_keys_select_sort_columns() {
        let mut app = App::with_processes(sample_processes());

        for (key, expected) in [
            ('1', SortColumn::Pid),
            ('2', SortColumn::Name),
            ('3', SortColumn::Cpu),
            ('4', SortColumn::Memory),
            ('5', SortColumn::Status),
        ] {
            app.on_key(KeyCode::Char(key), KeyModifiers::NONE);
            assert_eq!(app.sort_column, expected);
        }
    }

    #[test]
    fn test_unknown_key_is_ignored() {
        let mut app = App::with_processes(sample_processes());

        app.on_key(KeyCode::Char('z'), KeyModifiers::NONE);

        assert_eq!(app.mode, AppMode::Normal);
        assert!(!app.should_quit);
        assert_eq!(app.table_state.selected(), Some(0));
    }
}
