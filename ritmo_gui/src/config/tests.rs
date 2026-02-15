#[cfg(test)]
mod tests {
    use crate::config::GuiSettings;
    use crate::events::{TabState, Theme};

    #[test]
    fn test_gui_settings_default() {
        let settings = GuiSettings::default();
        assert_eq!(settings.last_tab, TabState::Books);
        assert_eq!(settings.theme, Theme::Dark);
        assert_eq!(settings.window_width, 1200.0);
        assert_eq!(settings.window_height, 800.0);
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
    fn test_theme_serialization() {
        // Theme is serialized as part of GuiSettings, not standalone
        let mut settings = GuiSettings::default();
        settings.theme = Theme::Light;
        
        let toml_str = toml::to_string(&settings).unwrap();
        let deserialized: GuiSettings = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.theme, Theme::Light);
    }
}
