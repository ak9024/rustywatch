use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, Terminal};
use sysinfo::System;
use std::{error::Error, io, time::{Duration, Instant}};

use crate::monitor::ui;

pub struct App {
    pub system: System,
    pub should_quit: bool,
    pub refresh_interval: Duration,
    pub last_refresh: Instant,
}

impl App {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        App {
            system,
            should_quit: false,
            refresh_interval: Duration::from_secs(1),
            last_refresh: Instant::now(),
        }
    }

    pub fn refresh(&mut self) {
        // Refresh process information
        self.system.refresh_processes();
        self.last_refresh = Instant::now();
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('r') => self.refresh(),
            _ => {}
        }
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    
    // Main loop
    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, &app))?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    app.on_key(key.code);
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
