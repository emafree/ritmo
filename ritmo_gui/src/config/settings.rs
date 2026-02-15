use crate::events::{TabState, Theme};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Persistent GUI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiSettings {
    /// Last active tab
    #[serde(default = "default_tab")]
    pub last_tab: TabState,
    
    /// Theme preference
    #[serde(default = "default_theme")]
    pub theme: Theme,
    
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
}

fn default_tab() -> TabState {
    TabState::Books
}

fn default_theme() -> Theme {
    Theme::Dark
}

fn default_window_size() -> f32 {
    800.0
}

impl Default for GuiSettings {
    fn default() -> Self {
        Self {
            last_tab: default_tab(),
            theme: default_theme(),
            window_width: 1200.0,
            window_height: 800.0,
            last_books_filter: None,
            last_contents_filter: None,
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
