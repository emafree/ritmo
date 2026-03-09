use slint::ComponentHandle;
use crate::{BookDialog, ContentSummary, FieldEntry, FieldDefinition};
use super::content_dialog::open_content_dialog_for_book;

pub fn open_book_dialog() -> anyhow::Result<()> {
    let dialog = BookDialog::new()?;

    dialog.set_available_field_names(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<slint::SharedString>::from(vec![])),
    ));
    dialog.set_available_fields(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<FieldDefinition>::from(vec![])),
    ));
    dialog.set_entries(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<FieldEntry>::from(vec![])),
    ));
    dialog.set_added_contents(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<ContentSummary>::from(vec![])),
    ));

    let dialog_weak = dialog.as_weak();
    dialog.on_cancelled(move || {
        if let Some(d) = dialog_weak.upgrade() {
            d.hide().unwrap();
        }
    });

    let dialog_weak2 = dialog.as_weak();
    dialog.on_accepted(move || {
        if let Some(d) = dialog_weak2.upgrade() {
            // TODO: collect entries and persist book to DB
            d.hide().unwrap();
        }
    });

    dialog.on_value_changed(move |_idx, _entry| {
        // TODO: update the entries model at index _idx
    });

    dialog.on_field_selected(move |_idx, _name| {
        // TODO: update field-type for the row at index _idx
    });

    dialog.on_delete_requested(move |_idx| {
        // TODO: remove the row at index _idx from entries model
    });

    // Open a nested ContentDialog when the user requests to add a content
    let dialog_weak3 = dialog.as_weak();
    dialog.on_add_content_requested(move || {
        let _ = open_content_dialog_for_book(dialog_weak3.clone());
    });

    let dialog_weak4 = dialog.as_weak();
    dialog.on_edit_content_requested(move |_idx| {
        if let Some(_d) = dialog_weak4.upgrade() {
            // TODO: open ContentDialog pre-filled with content at index _idx
        }
    });

    dialog.on_delete_content_requested(move |_idx| {
        // TODO: remove the content at index _idx from added-contents model
    });

    dialog.show()?;
    Ok(())
}
