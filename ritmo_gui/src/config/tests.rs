#[cfg(test)]
mod tests {
    use crate::config::{GuiSettings, ThemeMode, ThemePreset};
    use crate::events::TabState;

    #[test]
    fn test_gui_settings_default() {
        let settings = GuiSettings::default();
        assert_eq!(settings.last_tab, TabState::Books);
        assert!(matches!(settings.theme_mode, ThemeMode::Preset(ThemePreset::Dark)));
        assert_eq!(settings.window_width, 1200.0);
        assert_eq!(settings.window_height, 800.0);
        assert_eq!(settings.custom_themes.len(), 0);
    }

    #[test]
    fn test_tab_state_serialization() {
        use serde::{Serialize, Deserialize};
        
        let tab = TabState::Books;
        let json = serde_json::to_string(&tab).unwrap();
        let deserialized: TabState = serde_json::from_str(&json).unwrap();
        assert_eq!(tab, deserialized);
        
        let tab2 = TabState::Contents;
        let json2 = serde_json::to_string(&tab2).unwrap();
        let deserialized2: TabState = serde_json::from_str(&json2).unwrap();
        assert_eq!(tab2, deserialized2);
    }

    #[test]
    fn test_theme_mode_serialization() {
        let mut settings = GuiSettings::default();
        settings.theme_mode = ThemeMode::Preset(ThemePreset::Light);
        
        let toml_str = toml::to_string(&settings).unwrap();
        let deserialized: GuiSettings = toml::from_str(&toml_str).unwrap();
        assert!(matches!(deserialized.theme_mode, ThemeMode::Preset(ThemePreset::Light)));
    }
    
    #[test]
    fn test_custom_theme_management() {
        use crate::config::CustomTheme;
        
        let mut settings = GuiSettings::default();
        
        // Create a custom theme
        let theme = CustomTheme::from_preset(ThemePreset::Dark, "My Dark Theme".to_string());
        
        // Save it
        settings.save_custom_theme(theme.clone());
        assert_eq!(settings.custom_themes.len(), 1);
        assert_eq!(settings.custom_themes[0].name, "My Dark Theme");
        
        // Get it
        let retrieved = settings.get_custom_theme("My Dark Theme");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "My Dark Theme");
        
        // Delete it
        let deleted = settings.delete_custom_theme("My Dark Theme");
        assert!(deleted);
        assert_eq!(settings.custom_themes.len(), 0);
    }
}
