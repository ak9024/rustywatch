use ratatui::style::{Color, Modifier, Style};

/// Centralized color theme for the monitor UI
pub struct Theme {
    // Base colors
    pub fg: Color,

    // Accent colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,

    // Status colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // UI element colors
    pub border: Color,
    pub border_focused: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub muted: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // Base
            fg: Color::White,

            // Accent colors - using indexed colors for terminal compatibility
            primary: Color::Rgb(97, 175, 239),    // Soft blue
            secondary: Color::Rgb(152, 195, 121), // Soft green
            accent: Color::Rgb(229, 192, 123),    // Soft gold

            // Status colors
            success: Color::Rgb(152, 195, 121), // Green
            warning: Color::Rgb(229, 192, 123), // Yellow
            error: Color::Rgb(224, 108, 117),   // Red
            info: Color::Rgb(97, 175, 239),     // Blue

            // UI elements
            border: Color::Rgb(92, 99, 112),        // Muted gray
            border_focused: Color::Rgb(97, 175, 239), // Blue when focused
            header_bg: Color::Rgb(40, 44, 52),      // Dark gray
            header_fg: Color::Rgb(171, 178, 191),   // Light gray
            selected_bg: Color::Rgb(62, 68, 81),    // Slightly lighter dark
            selected_fg: Color::White,
            muted: Color::Rgb(92, 99, 112), // Gray for less important text
        }
    }
}

impl Theme {
    /// Style for the main title
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for table headers
    pub fn header_style(&self) -> Style {
        Style::default()
            .fg(self.header_fg)
            .bg(self.header_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for selected rows
    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.selected_fg)
            .bg(self.selected_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for borders
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Style for focused borders
    pub fn border_focused_style(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    /// Get status color based on value thresholds
    pub fn status_color(&self, value: f32, warning_threshold: f32, error_threshold: f32) -> Color {
        if value >= error_threshold {
            self.error
        } else if value >= warning_threshold {
            self.warning
        } else {
            self.success
        }
    }

    /// Get process status style
    pub fn process_status_style(&self, status: &str) -> Style {
        match status.to_lowercase().as_str() {
            "run" | "running" => Style::default().fg(self.success),
            "sleep" | "sleeping" | "idle" => Style::default().fg(self.info),
            "stop" | "stopped" => Style::default().fg(self.warning),
            "zombie" | "dead" => Style::default().fg(self.error),
            _ => Style::default().fg(self.muted),
        }
    }

    /// Get status indicator symbol
    pub fn status_indicator(&self, status: &str) -> &'static str {
        match status.to_lowercase().as_str() {
            "run" | "running" => "●",
            "sleep" | "sleeping" | "idle" => "◐",
            "stop" | "stopped" => "○",
            "zombie" | "dead" => "◌",
            _ => "·",
        }
    }
}
