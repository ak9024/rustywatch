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
            border: Color::Rgb(92, 99, 112),          // Muted gray
            border_focused: Color::Rgb(97, 175, 239), // Blue when focused
            header_bg: Color::Rgb(40, 44, 52),        // Dark gray
            header_fg: Color::Rgb(171, 178, 191),     // Light gray
            selected_bg: Color::Rgb(62, 68, 81),      // Slightly lighter dark
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_color_thresholds() {
        let theme = Theme::default();

        assert_eq!(theme.status_color(10.0, 60.0, 85.0), theme.success);
        assert_eq!(theme.status_color(70.0, 60.0, 85.0), theme.warning);
        assert_eq!(theme.status_color(90.0, 60.0, 85.0), theme.error);
    }

    // The comparisons are `>=`, so a value sitting exactly on a threshold must
    // take the more severe colour.
    #[test]
    fn test_status_color_is_inclusive_at_thresholds() {
        let theme = Theme::default();

        assert_eq!(theme.status_color(60.0, 60.0, 85.0), theme.warning);
        assert_eq!(theme.status_color(85.0, 60.0, 85.0), theme.error);
    }

    #[test]
    fn test_process_status_style_matches_sysinfo_casing() {
        let theme = Theme::default();

        // `sysinfo` renders statuses like "Run"/"Sleep", so matching is
        // case-insensitive on purpose.
        for (status, expected) in [
            ("Run", theme.success),
            ("running", theme.success),
            ("Sleep", theme.info),
            ("idle", theme.info),
            ("Stop", theme.warning),
            ("stopped", theme.warning),
            ("Zombie", theme.error),
            ("dead", theme.error),
            ("Unknown", theme.muted),
        ] {
            assert_eq!(
                theme.process_status_style(status).fg,
                Some(expected),
                "unexpected colour for status `{status}`"
            );
        }
    }

    #[test]
    fn test_status_indicator() {
        let theme = Theme::default();

        assert_eq!(theme.status_indicator("Run"), "●");
        assert_eq!(theme.status_indicator("sleeping"), "◐");
        assert_eq!(theme.status_indicator("Stop"), "○");
        assert_eq!(theme.status_indicator("zombie"), "◌");
        assert_eq!(theme.status_indicator("whatever"), "·");
    }

    #[test]
    fn test_styles_carry_theme_colours() {
        let theme = Theme::default();

        assert_eq!(theme.title_style().fg, Some(theme.primary));
        assert!(theme.title_style().add_modifier.contains(Modifier::BOLD));

        assert_eq!(theme.header_style().fg, Some(theme.header_fg));
        assert_eq!(theme.header_style().bg, Some(theme.header_bg));

        assert_eq!(theme.selected_style().fg, Some(theme.selected_fg));
        assert_eq!(theme.selected_style().bg, Some(theme.selected_bg));

        assert_eq!(theme.border_style().fg, Some(theme.border));
        assert_eq!(theme.border_focused_style().fg, Some(theme.border_focused));
    }
}
