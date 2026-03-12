use crate::util::{dark_defaults as D, parse_hex_color};
use crate::{MainWindow, OptionsDialog, Theme};
use slint::ComponentHandle;

/// Costruisce una `ColorPalette` dai valori hex del dialog.
fn build_color_palette(d: &OptionsDialog) -> crate::ColorPalette {
    crate::ColorPalette {
        bg: parse_hex_color(&d.get_custom_bg()),
        surface: parse_hex_color(&d.get_custom_surface()),
        surface2: parse_hex_color(&d.get_custom_surface2()),
        surface3: parse_hex_color(&d.get_custom_surface3()),
        border: parse_hex_color(&d.get_custom_border()),
        border2: parse_hex_color(&d.get_custom_border2()),
        text_primary: parse_hex_color(&d.get_custom_text_primary()),
        text_secondary: parse_hex_color(&d.get_custom_text_secondary()),
        text_muted: parse_hex_color(&d.get_custom_text_muted()),
        accent: parse_hex_color(&d.get_custom_accent()),
        accent2: parse_hex_color(&d.get_custom_accent2()),
        active_bg: parse_hex_color(&d.get_custom_active_bg()),
        tag_bg: parse_hex_color(&d.get_custom_tag_bg()),
        tag_text: parse_hex_color(&d.get_custom_tag_text()),
        danger: parse_hex_color(&d.get_custom_danger()),
        success: parse_hex_color(&d.get_custom_success()),
    }
}

pub fn open_options_dialog(win: &MainWindow) -> anyhow::Result<()> {
    let dialog = OptionsDialog::new()?;

    // Salva il valore corrente per eventuale ripristino al cancel
    let current_palette = win.global::<Theme>().get_active_palette();
    dialog.set_active_palette(current_palette);

    // Pre-popola i campi hex della palette custom dai settings.toml
    let settings_path = ritmo_config::settings_file()?;
    let app_settings =
        ritmo_config::AppSettings::load_or_create(&settings_path).unwrap_or_default();
    let cp = &app_settings.preferences.custom_palette;

    dialog.set_custom_bg(cp.bg.clone().unwrap_or_else(|| D::BG.to_string()).into());
    dialog.set_custom_surface(
        cp.surface.clone().unwrap_or_else(|| D::SURFACE.to_string()).into(),
    );
    dialog.set_custom_surface2(
        cp.surface2.clone().unwrap_or_else(|| D::SURFACE2.to_string()).into(),
    );
    dialog.set_custom_surface3(
        cp.surface3.clone().unwrap_or_else(|| D::SURFACE3.to_string()).into(),
    );
    dialog.set_custom_border(
        cp.border.clone().unwrap_or_else(|| D::BORDER.to_string()).into(),
    );
    dialog.set_custom_border2(
        cp.border2.clone().unwrap_or_else(|| D::BORDER2.to_string()).into(),
    );
    dialog.set_custom_text_primary(
        cp.text_primary.clone().unwrap_or_else(|| D::TEXT_PRIMARY.to_string()).into(),
    );
    dialog.set_custom_text_secondary(
        cp.text_secondary
            .clone()
            .unwrap_or_else(|| D::TEXT_SECONDARY.to_string())
            .into(),
    );
    dialog.set_custom_text_muted(
        cp.text_muted.clone().unwrap_or_else(|| D::TEXT_MUTED.to_string()).into(),
    );
    dialog.set_custom_accent(
        cp.accent.clone().unwrap_or_else(|| D::ACCENT.to_string()).into(),
    );
    dialog.set_custom_accent2(
        cp.accent2.clone().unwrap_or_else(|| D::ACCENT2.to_string()).into(),
    );
    dialog.set_custom_active_bg(
        cp.active_bg.clone().unwrap_or_else(|| D::ACTIVE_BG.to_string()).into(),
    );
    dialog.set_custom_tag_bg(
        cp.tag_bg.clone().unwrap_or_else(|| D::TAG_BG.to_string()).into(),
    );
    dialog.set_custom_tag_text(
        cp.tag_text.clone().unwrap_or_else(|| D::TAG_TEXT.to_string()).into(),
    );
    dialog.set_custom_danger(
        cp.danger.clone().unwrap_or_else(|| D::DANGER.to_string()).into(),
    );
    dialog.set_custom_success(
        cp.success.clone().unwrap_or_else(|| D::SUCCESS.to_string()).into(),
    );

    // ── on_accepted: salva e applica ─────────────────────────────────────
    let win_weak = win.as_weak();
    let dialog_weak = dialog.as_weak();
    dialog.on_accepted(move || {
        let (Some(d), Some(w)) = (dialog_weak.upgrade(), win_weak.upgrade()) else {
            return;
        };

        let palette_idx = d.get_active_palette();
        let theme_str = match palette_idx {
            1 => "light",
            2 => "custom",
            _ => "dark",
        };

        // Salva in settings.toml
        if let Ok(path) = ritmo_config::settings_file() {
            if let Ok(mut settings) = ritmo_config::AppSettings::load_or_create(&path) {
                settings.set_theme(theme_str.to_string());

                if palette_idx == 2 {
                    settings.preferences.custom_palette.bg =
                        Some(d.get_custom_bg().to_string());
                    settings.preferences.custom_palette.surface =
                        Some(d.get_custom_surface().to_string());
                    settings.preferences.custom_palette.surface2 =
                        Some(d.get_custom_surface2().to_string());
                    settings.preferences.custom_palette.surface3 =
                        Some(d.get_custom_surface3().to_string());
                    settings.preferences.custom_palette.border =
                        Some(d.get_custom_border().to_string());
                    settings.preferences.custom_palette.border2 =
                        Some(d.get_custom_border2().to_string());
                    settings.preferences.custom_palette.text_primary =
                        Some(d.get_custom_text_primary().to_string());
                    settings.preferences.custom_palette.text_secondary =
                        Some(d.get_custom_text_secondary().to_string());
                    settings.preferences.custom_palette.text_muted =
                        Some(d.get_custom_text_muted().to_string());
                    settings.preferences.custom_palette.accent =
                        Some(d.get_custom_accent().to_string());
                    settings.preferences.custom_palette.accent2 =
                        Some(d.get_custom_accent2().to_string());
                    settings.preferences.custom_palette.active_bg =
                        Some(d.get_custom_active_bg().to_string());
                    settings.preferences.custom_palette.tag_bg =
                        Some(d.get_custom_tag_bg().to_string());
                    settings.preferences.custom_palette.tag_text =
                        Some(d.get_custom_tag_text().to_string());
                    settings.preferences.custom_palette.danger =
                        Some(d.get_custom_danger().to_string());
                    settings.preferences.custom_palette.success =
                        Some(d.get_custom_success().to_string());
                }

                let _ = settings.save(&path);
            }
        }

        // Aggiorna il tema live nella finestra principale
        w.global::<Theme>().set_active_palette(palette_idx);
        if palette_idx == 2 {
            w.global::<Theme>().set_custom_palette(build_color_palette(&d));
        }

        d.hide().unwrap();
    });

    // ── on_cancelled: ripristina e chiudi ────────────────────────────────
    let win_weak2 = win.as_weak();
    let dialog_weak2 = dialog.as_weak();
    dialog.on_cancelled(move || {
        // Ripristina il palette index originale nella finestra principale
        if let Some(w) = win_weak2.upgrade() {
            w.global::<Theme>().set_active_palette(current_palette);
        }
        if let Some(d) = dialog_weak2.upgrade() {
            d.hide().unwrap();
        }
    });

    dialog.show()?;
    Ok(())
}
