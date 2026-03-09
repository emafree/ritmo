use slint::{ComponentHandle, Model};
use crate::{ContentDialog, FieldEntry, FieldDefinition};

fn init_dialog(dialog: &ContentDialog) {
    dialog.set_languages(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<slint::SharedString>::from(vec![])),
    ));
    dialog.set_available_field_names(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<slint::SharedString>::from(vec![])),
    ));
    dialog.set_available_fields(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<FieldDefinition>::from(vec![])),
    ));
    dialog.set_entries(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<FieldEntry>::from(vec![])),
    ));
}

fn wire_field_callbacks(dialog: &ContentDialog) {
    dialog.on_value_changed(move |_idx, _entry| {
        // TODO: update the entries model at index _idx
    });

    dialog.on_field_selected(move |_idx, _name| {
        // TODO: update field-type for the row at index _idx
    });

    dialog.on_delete_requested(move |_idx| {
        // TODO: remove the row at index _idx from entries model
    });
}

pub fn open_content_dialog() -> anyhow::Result<()> {
    let dialog = ContentDialog::new()?;
    init_dialog(&dialog);

    let dialog_weak = dialog.as_weak();
    dialog.on_cancelled(move || {
        if let Some(d) = dialog_weak.upgrade() {
            d.hide().unwrap();
        }
    });

    let dialog_weak2 = dialog.as_weak();
    dialog.on_accepted(move || {
        if let Some(d) = dialog_weak2.upgrade() {
            // TODO: collect entries and persist
            d.hide().unwrap();
        }
    });

    wire_field_callbacks(&dialog);
    dialog.show()?;
    Ok(())
}

/// Opens a `ContentDialog` nested inside a `BookDialog`, wiring the `accepted`
/// callback so that the new content summary is appended to the book's list.
pub fn open_content_dialog_for_book(
    book_dialog_weak: slint::Weak<crate::BookDialog>,
) -> anyhow::Result<()> {
    let dialog = ContentDialog::new()?;
    init_dialog(&dialog);

    let dialog_weak = dialog.as_weak();
    dialog.on_cancelled(move || {
        if let Some(d) = dialog_weak.upgrade() {
            d.hide().unwrap();
        }
    });

    let dialog_weak2 = dialog.as_weak();
    dialog.on_accepted(move || {
        if let Some(d) = dialog_weak2.upgrade() {
            if let Some(book) = book_dialog_weak.upgrade() {
                // Collect a simple summary from ContentDialog's own entries
                let entries = d.get_entries();
                let title = if entries.row_count() > 0 {
                    entries.row_data(0).map(|e| e.value).unwrap_or_default()
                } else {
                    slint::SharedString::default()
                };
                let new_content = crate::ContentSummary {
                    title,
                    author: slint::SharedString::default(),
                };
                let current = book.get_added_contents();
                let mut items: Vec<crate::ContentSummary> =
                    (0..current.row_count()).filter_map(|i| current.row_data(i)).collect();
                items.push(new_content);
                book.set_added_contents(slint::ModelRc::from(
                    std::rc::Rc::new(slint::VecModel::from(items)),
                ));
            }
            d.hide().unwrap();
        }
    });

    wire_field_callbacks(&dialog);
    dialog.show()?;
    Ok(())
}

