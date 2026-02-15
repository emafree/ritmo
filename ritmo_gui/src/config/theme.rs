use crate::events::Theme;
use egui::{Color32, Visuals};

/// Theme configuration for the GUI
pub struct ThemeConfig;

impl ThemeConfig {
    /// Get egui Visuals for the given theme
    pub fn get_visuals(theme: Theme) -> Visuals {
        match theme {
            Theme::Dark => Visuals::dark(),
            Theme::Light => Visuals::light(),
        }
    }
    
    /// Get accent color for the theme
    pub fn get_accent_color(theme: Theme) -> Color32 {
        match theme {
            Theme::Dark => Color32::from_rgb(100, 150, 255),
            Theme::Light => Color32::from_rgb(50, 100, 200),
        }
    }
}
