use egui::Color32;
use crate::config::{ThemeConfig, ThemeMode, CustomTheme};

/// Derived UI colour palette built on top of the active egui visuals.
///
/// All extra colours (surface2/3, border, accent, text2/3 …) are computed
/// automatically from the theme so the user does not need extra settings fields.
pub struct UiPalette {
    pub bg: Color32,
    pub surface: Color32,
    pub surface2: Color32,
    pub surface3: Color32,
    pub border: Color32,
    pub border2: Color32,
    pub text: Color32,
    pub text2: Color32,
    pub text3: Color32,
    pub accent: Color32,
    pub accent2: Color32,
    /// Highlight fill for the active/selected filter header
    pub active: Color32,
    /// Tag background (subtle tinted fill)
    pub tag: Color32,
    /// Tag text colour
    pub tag_text: Color32,
}

impl UiPalette {
    /// Derive the palette from the currently applied egui visuals and the accent
    /// colour from the theme config.  Call after `ctx.set_visuals(...)`.
    pub fn from_visuals(visuals: &egui::Visuals, accent: Color32) -> Self {
        let bg = visuals.extreme_bg_color;
        let surface = visuals.widgets.noninteractive.bg_fill;

        let lighten = |c: Color32, amount: u8| -> Color32 {
            Color32::from_rgb(
                c.r().saturating_add(amount),
                c.g().saturating_add(amount),
                c.b().saturating_add(amount),
            )
        };
        let darken = |c: Color32, amount: u8| -> Color32 {
            Color32::from_rgb(
                c.r().saturating_sub(amount),
                c.g().saturating_sub(amount),
                c.b().saturating_sub(amount),
            )
        };

        let is_dark = visuals.dark_mode;

        let surface2 = if is_dark { lighten(surface, 12) } else { darken(surface, 8) };
        let surface3 = if is_dark { lighten(surface, 24) } else { darken(surface, 16) };

        let border = if is_dark { lighten(bg, 28) } else { darken(bg, 24) };
        let border2 = if is_dark { lighten(bg, 40) } else { darken(bg, 36) };

        let text = visuals.text_color();
        let text2 = Color32::from_rgba_unmultiplied(text.r(), text.g(), text.b(), 153); // ~60%
        let text3 = Color32::from_rgba_unmultiplied(text.r(), text.g(), text.b(), 85);  // ~33%

        // active: slightly tinted version of accent mixed with surface2
        let active = Color32::from_rgb(
            ((accent.r() as u16 * 30 + surface2.r() as u16 * 70) / 100) as u8,
            ((accent.g() as u16 * 30 + surface2.g() as u16 * 70) / 100) as u8,
            ((accent.b() as u16 * 30 + surface2.b() as u16 * 70) / 100) as u8,
        );

        let accent2 = darken(accent, 30);

        // tag: green-tinted surface
        let tag = if is_dark {
            Color32::from_rgb(
                surface2.r().saturating_sub(10),
                surface2.g().saturating_add(15),
                surface2.b().saturating_sub(10),
            )
        } else {
            Color32::from_rgb(
                surface2.r().saturating_sub(5),
                surface2.g().saturating_add(10),
                surface2.b().saturating_sub(5),
            )
        };
        let tag_text = Color32::from_rgb(122, 184, 122);

        Self {
            bg,
            surface,
            surface2,
            surface3,
            border,
            border2,
            text,
            text2,
            text3,
            accent,
            accent2,
            active,
            tag,
            tag_text,
        }
    }

    /// Build the palette directly from the app's theme settings.
    pub fn from_settings(
        theme_mode: &ThemeMode,
        custom_themes: &[CustomTheme],
    ) -> Self {
        let visuals = ThemeConfig::get_visuals(theme_mode, custom_themes);
        let accent = ThemeConfig::get_accent_color(theme_mode, custom_themes);
        Self::from_visuals(&visuals, accent)
    }
}
