use slint::ComponentHandle;
use crate::FilterDialog;

pub fn open_filter_dialog() -> anyhow::Result<()> {
    let dialog = FilterDialog::new()?;

    let dialog_weak = dialog.as_weak();
    dialog.on_cancelled(move || {
        if let Some(d) = dialog_weak.upgrade() {
            d.hide().unwrap();
        }
    });

    let dialog_weak2 = dialog.as_weak();
    dialog.on_accepted(move || {
        if let Some(d) = dialog_weak2.upgrade() {
            // TODO: collect filter rows and apply filtering
            d.hide().unwrap();
        }
    });

    dialog.show()?;
    Ok(())
}
