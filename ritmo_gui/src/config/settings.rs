use crate::events::TabState;
use crate::config::theme::{ThemeMode, CustomTheme};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Book list display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewMode {
    /// Scrollable list (default)
    List,
    /// Scrollable grid with thumbnail cards
    Grid,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::List
    }
}

fn default_view_mode() -> ViewMode {
    ViewMode::default()
}

/// Persistent GUI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiSettings {
    /// Last active tab
    #[serde(default = "default_tab")]
    pub last_tab: TabState,
    
    /// Theme mode (preset or custom)
    #[serde(default = "default_theme_mode")]
    pub theme_mode: ThemeMode,
    
    /// Custom themes
    #[serde(default)]
    pub custom_themes: Vec<CustomTheme>,
    
    /// Window size
    #[serde(default = "default_window_size")]
    pub window_width: f32,
    
    #[serde(default = "default_window_size")]
    pub window_height: f32,
    
    /// Last used filters (serialized as JSON strings)
    #[serde(default)]
    pub last_books_filter: Option<String>,
    
    #[serde(default)]
    pub last_contents_filter: Option<String>,
    
    /// View mode for the books tab (list or grid)
    #[serde(default = "default_view_mode")]
    pub view_mode: ViewMode,
}

fn default_tab() -> TabState {
    TabState::Books
}

fn default_theme_mode() -> ThemeMode {
    ThemeMode::default()
}

fn default_window_size() -> f32 {
    800.0
}

impl Default for GuiSettings {
    fn default() -> Self {
        Self {
            last_tab: default_tab(),
            theme_mode: default_theme_mode(),
            custom_themes: Vec::new(),
            window_width: 1200.0,
            window_height: 800.0,
            last_books_filter: None,
            last_contents_filter: None,
            view_mode: default_view_mode(),
        }
    }
}

impl GuiSettings {
    /// Load settings from config file
    pub fn load() -> Self {
        let path = Self::config_path();
        
        if let Ok(contents) = std::fs::read_to_string(&path) {
            toml::from_str(&contents).unwrap_or_default()
        } else {
            Self::default()
        }
    }
    
    /// Save settings to config file
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        
        Ok(())
    }
    
    /// Get the config file path
    fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".ritmo")
            .join("gui_config.toml")
    }
    
    /// Add or update a custom theme
    pub fn save_custom_theme(&mut self, theme: CustomTheme) {
        // Check if theme with this name already exists
        if let Some(existing) = self.custom_themes.iter_mut().find(|t| t.name == theme.name) {
            *existing = theme;
        } else {
            self.custom_themes.push(theme);
        }
    }
    
    /// Delete a custom theme by name
    pub fn delete_custom_theme(&mut self, name: &str) -> bool {
        let initial_len = self.custom_themes.len();
        self.custom_themes.retain(|t| t.name != name);
        self.custom_themes.len() != initial_len
    }
    
    /// Get a custom theme by name
    pub fn get_custom_theme(&self, name: &str) -> Option<&CustomTheme> {
        self.custom_themes.iter().find(|t| t.name == name)
    }
}

// Implement Serialize/Deserialize for TabState
impl Serialize for TabState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            TabState::Books => "Books",
            TabState::Contents => "Contents",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for TabState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Books" => Ok(TabState::Books),
            "Contents" => Ok(TabState::Contents),
            _ => Ok(TabState::Books),
        }
    }
}
