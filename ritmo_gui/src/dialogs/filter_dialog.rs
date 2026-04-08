use slint::ComponentHandle;
use crate::FilterDialog;

pub fn open_filter_dialog(main_win: &crate::MainWindow) -> anyhow::Result<()> {
    let dialog = FilterDialog::new()?;

    // Sync theme from the main window
    let active_palette = main_win.global::<crate::Theme>().get_active_palette();
    dialog.global::<crate::Theme>().set_active_palette(active_palette);
    if active_palette == 2 {
        let custom_palette = main_win.global::<crate::Theme>().get_custom_palette();
        dialog.global::<crate::Theme>().set_custom_palette(custom_palette);
    }

    let dialog_weak = dialog.as_weak();
    dialog.on_cancelled(move || {
        if let Some(d) = dialog_weak.upgrade() {
            d.hide().unwrap();
        }
    });

    // "Continua": saves the current row as part of the filter and keeps the dialog open
    let dialog_weak2 = dialog.as_weak();
    dialog.on_continued(move || {
        if let Some(_d) = dialog_weak2.upgrade() {
            // TODO: collect and store the current filter row, then reset for next row
        }
    });

    let dialog_weak3 = dialog.as_weak();
    dialog.on_accepted(move || {
        if let Some(d) = dialog_weak3.upgrade() {
            // TODO: collect filter rows and apply filtering
            d.hide().unwrap();
        }
    });

    dialog.show()?;
    Ok(())
}
