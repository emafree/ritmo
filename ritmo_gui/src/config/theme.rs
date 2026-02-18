use egui::{Color32, Visuals};
use serde::{Deserialize, Serialize};

/// Predefined theme presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreset {
    Dark,
    Light,
    SolarizedDark,
    SolarizedLight,
    Nord,
    Dracula,
}

impl ThemePreset {
    pub fn name(&self) -> &str {
        match self {
            ThemePreset::Dark => "Dark",
            ThemePreset::Light => "Light",
            ThemePreset::SolarizedDark => "Solarized Dark",
            ThemePreset::SolarizedLight => "Solarized Light",
            ThemePreset::Nord => "Nord",
            ThemePreset::Dracula => "Dracula",
        }
    }
    
    pub fn all() -> &'static [ThemePreset] {
        &[
            ThemePreset::Dark,
            ThemePreset::Light,
            ThemePreset::SolarizedDark,
            ThemePreset::SolarizedLight,
            ThemePreset::Nord,
            ThemePreset::Dracula,
        ]
    }
}

/// Custom theme with full color palette
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTheme {
    pub name: String,
    pub description: Option<String>,
    pub background: [u8; 3],
    pub foreground: [u8; 3],
    pub accent: [u8; 3],
    pub success: [u8; 3],
    pub warning: [u8; 3],
    pub error: [u8; 3],
    pub text_color: [u8; 3],
    pub weak_text_color: [u8; 3],
    pub created_at: i64,
    pub updated_at: i64,
}

impl CustomTheme {
    /// Create a new custom theme from a preset as template
    pub fn from_preset(preset: ThemePreset, name: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        let (bg, fg, accent, success, warning, error, text, weak_text) = 
            ThemeConfig::get_preset_colors(preset);
        
        Self {
            name,
            description: None,
            background: [bg.r(), bg.g(), bg.b()],
            foreground: [fg.r(), fg.g(), fg.b()],
            accent: [accent.r(), accent.g(), accent.b()],
            success: [success.r(), success.g(), success.b()],
            warning: [warning.r(), warning.g(), warning.b()],
            error: [error.r(), error.g(), error.b()],
            text_color: [text.r(), text.g(), text.b()],
            weak_text_color: [weak_text.r(), weak_text.g(), weak_text.b()],
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Update the timestamp
    pub fn update_timestamp(&mut self) {
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

/// Theme mode - either a preset or a custom theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeMode {
    Preset(ThemePreset),
    Custom(String), // Name of the custom theme
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Preset(ThemePreset::Dark)
    }
}

/// Theme configuration for the GUI
pub struct ThemeConfig;

impl ThemeConfig {
    /// Get egui Visuals for the given theme mode
    pub fn get_visuals(theme_mode: &ThemeMode, custom_themes: &[CustomTheme]) -> Visuals {
        match theme_mode {
            ThemeMode::Preset(preset) => Self::get_preset_visuals(*preset),
            ThemeMode::Custom(name) => {
                if let Some(theme) = custom_themes.iter().find(|t| &t.name == name) {
                    Self::get_custom_visuals(theme)
                } else {
                    // Fallback to dark if custom theme not found
                    Self::get_preset_visuals(ThemePreset::Dark)
                }
            }
        }
    }
    
    /// Get egui Visuals for a preset theme
    fn get_preset_visuals(preset: ThemePreset) -> Visuals {
        match preset {
            ThemePreset::Dark => Visuals::dark(),
            ThemePreset::Light => Visuals::light(),
            ThemePreset::SolarizedDark => {
                let mut visuals = Visuals::dark();
                visuals.override_text_color = Some(Color32::from_rgb(131, 148, 150));
                visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(0, 43, 54);
                visuals.extreme_bg_color = Color32::from_rgb(0, 43, 54);
                visuals.faint_bg_color = Color32::from_rgb(7, 54, 66);
                visuals
            }
            ThemePreset::SolarizedLight => {
                let mut visuals = Visuals::light();
                visuals.override_text_color = Some(Color32::from_rgb(101, 123, 131));
                visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(253, 246, 227);
                visuals.extreme_bg_color = Color32::from_rgb(253, 246, 227);
                visuals.faint_bg_color = Color32::from_rgb(238, 232, 213);
                visuals
            }
            ThemePreset::Nord => {
                let mut visuals = Visuals::dark();
                visuals.override_text_color = Some(Color32::from_rgb(216, 222, 233));
                visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(46, 52, 64);
                visuals.extreme_bg_color = Color32::from_rgb(46, 52, 64);
                visuals.faint_bg_color = Color32::from_rgb(59, 66, 82);
                visuals
            }
            ThemePreset::Dracula => {
                let mut visuals = Visuals::dark();
                visuals.override_text_color = Some(Color32::from_rgb(248, 248, 242));
                visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(40, 42, 54);
                visuals.extreme_bg_color = Color32::from_rgb(40, 42, 54);
                visuals.faint_bg_color = Color32::from_rgb(68, 71, 90);
                visuals
            }
        }
    }
    
    /// Get egui Visuals for a custom theme
    fn get_custom_visuals(theme: &CustomTheme) -> Visuals {
        let mut visuals = Visuals::dark();
        let bg = Color32::from_rgb(theme.background[0], theme.background[1], theme.background[2]);
        let text = Color32::from_rgb(theme.text_color[0], theme.text_color[1], theme.text_color[2]);
        
        visuals.override_text_color = Some(text);
        visuals.widgets.noninteractive.bg_fill = bg;
        visuals.extreme_bg_color = bg;
        visuals.faint_bg_color = Color32::from_rgb(
            theme.foreground[0],
            theme.foreground[1],
            theme.foreground[2],
        );
        
        visuals
    }
    
    /// Get accent color for the theme
    pub fn get_accent_color(theme_mode: &ThemeMode, custom_themes: &[CustomTheme]) -> Color32 {
        match theme_mode {
            ThemeMode::Preset(preset) => {
                let (_, _, accent, _, _, _, _, _) = Self::get_preset_colors(*preset);
                accent
            }
            ThemeMode::Custom(name) => {
                if let Some(theme) = custom_themes.iter().find(|t| &t.name == name) {
                    Color32::from_rgb(theme.accent[0], theme.accent[1], theme.accent[2])
                } else {
                    Color32::from_rgb(100, 150, 255)
                }
            }
        }
    }
    
    /// Get preset theme colors (bg, fg, accent, success, warning, error, text, weak_text)
    fn get_preset_colors(preset: ThemePreset) -> (Color32, Color32, Color32, Color32, Color32, Color32, Color32, Color32) {
        match preset {
            ThemePreset::Dark => (
                Color32::from_rgb(27, 27, 27),
                Color32::from_rgb(50, 50, 50),
                Color32::from_rgb(100, 150, 255),
                Color32::from_rgb(100, 200, 100),
                Color32::from_rgb(255, 200, 100),
                Color32::from_rgb(255, 100, 100),
                Color32::from_rgb(220, 220, 220),
                Color32::from_rgb(140, 140, 140),
            ),
            ThemePreset::Light => (
                Color32::from_rgb(248, 248, 248),
                Color32::from_rgb(230, 230, 230),
                Color32::from_rgb(50, 100, 200),
                Color32::from_rgb(50, 150, 50),
                Color32::from_rgb(200, 150, 50),
                Color32::from_rgb(200, 50, 50),
                Color32::from_rgb(30, 30, 30),
                Color32::from_rgb(100, 100, 100),
            ),
            ThemePreset::SolarizedDark => (
                Color32::from_rgb(0, 43, 54),
                Color32::from_rgb(7, 54, 66),
                Color32::from_rgb(38, 139, 210),
                Color32::from_rgb(133, 153, 0),
                Color32::from_rgb(181, 137, 0),
                Color32::from_rgb(220, 50, 47),
                Color32::from_rgb(131, 148, 150),
                Color32::from_rgb(88, 110, 117),
            ),
            ThemePreset::SolarizedLight => (
                Color32::from_rgb(253, 246, 227),
                Color32::from_rgb(238, 232, 213),
                Color32::from_rgb(38, 139, 210),
                Color32::from_rgb(133, 153, 0),
                Color32::from_rgb(181, 137, 0),
                Color32::from_rgb(220, 50, 47),
                Color32::from_rgb(101, 123, 131),
                Color32::from_rgb(147, 161, 161),
            ),
            ThemePreset::Nord => (
                Color32::from_rgb(46, 52, 64),
                Color32::from_rgb(59, 66, 82),
                Color32::from_rgb(136, 192, 208),
                Color32::from_rgb(163, 190, 140),
                Color32::from_rgb(235, 203, 139),
                Color32::from_rgb(191, 97, 106),
                Color32::from_rgb(216, 222, 233),
                Color32::from_rgb(143, 157, 177),
            ),
            ThemePreset::Dracula => (
                Color32::from_rgb(40, 42, 54),
                Color32::from_rgb(68, 71, 90),
                Color32::from_rgb(189, 147, 249),
                Color32::from_rgb(80, 250, 123),
                Color32::from_rgb(241, 250, 140),
                Color32::from_rgb(255, 85, 85),
                Color32::from_rgb(248, 248, 242),
                Color32::from_rgb(98, 114, 164),
            ),
        }
    }
}
