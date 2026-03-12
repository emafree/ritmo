// ritmo_gui/src/util.rs — utility functions condivise

/// Parsa un colore hex nel formato "#rrggbb" in un `slint::Color`.
pub fn parse_hex_color(hex: &str) -> slint::Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return slint::Color::from_rgb_u8(0, 0, 0);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    slint::Color::from_rgb_u8(r, g, b)
}

/// Valori hex di default della DarkPalette (usati come fallback)
pub mod dark_defaults {
    pub const BG: &str = "#0f0f0f";
    pub const SURFACE: &str = "#171717";
    pub const SURFACE2: &str = "#1f1f1f";
    pub const SURFACE3: &str = "#282828";
    pub const BORDER: &str = "#2e2e2e";
    pub const BORDER2: &str = "#3a3a3a";
    pub const TEXT_PRIMARY: &str = "#e8e8e8";
    pub const TEXT_SECONDARY: &str = "#999999";
    pub const TEXT_MUTED: &str = "#555555";
    pub const ACCENT: &str = "#c8a96e";
    pub const ACCENT2: &str = "#a07040";
    pub const ACTIVE_BG: &str = "#2a2218";
    pub const TAG_BG: &str = "#1e2a1e";
    pub const TAG_TEXT: &str = "#7ab87a";
    pub const DANGER: &str = "#c0392b";
    pub const SUCCESS: &str = "#27ae60";
}
