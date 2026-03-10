use super::content_dialog::open_content_dialog_for_book;
use crate::{BookDialog, ContentSummary, FieldDefinition, FieldEntry};
use slint::{ComponentHandle, Model};

pub fn open_book_dialog() -> anyhow::Result<()> {
    let dialog = BookDialog::new()?;

    dialog.set_available_field_names(slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::<
        slint::SharedString,
    >::from(vec![]))));
    dialog.set_available_fields(slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::<
        FieldDefinition,
    >::from(vec![]))));
    // Initial placeholder row so the UI shows an empty field ready for input
    dialog.set_entries(slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::<
        FieldEntry,
    >::from(vec![
        FieldEntry {
            field_name: slint::SharedString::default(),
            field_type: slint::SharedString::from("text"),
            value: slint::SharedString::default(),
        },
    ]))));
    dialog.set_added_contents(slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::<
        ContentSummary,
    >::from(vec![]))));

    let dialog_weak = dialog.as_weak();
    dialog.on_cancelled(move || {
        if let Some(d) = dialog_weak.upgrade() {
            d.hide().unwrap();
        }
    });

    let dialog_weak2 = dialog.as_weak();
    dialog.on_accepted(move || {
        if let Some(d) = dialog_weak2.upgrade() {
            let entries = d.get_entries();
            let _book_entries: Vec<FieldEntry> = (0..entries.row_count())
                .filter_map(|i| entries.row_data(i))
                .filter(|e| !e.field_name.is_empty())
                .collect();
            // TODO: persist _book_entries to DB
            d.hide().unwrap();
        }
    });

    let dialog_weak3 = dialog.as_weak();
    dialog.on_value_changed(move |idx, entry| {
        if let Some(d) = dialog_weak3.upgrade() {
            let entries = d.get_entries();
            let model = entries
                .as_any()
                .downcast_ref::<slint::VecModel<FieldEntry>>()
                .expect("entries must be VecModel<FieldEntry>");

            model.set_row_data(idx as usize, entry.clone());

            // When the user fills the last (placeholder) row, append a new empty placeholder
            if !entry.value.is_empty() && idx as usize == model.row_count() - 1 {
                model.push(FieldEntry {
                    field_name: slint::SharedString::default(),
                    field_type: slint::SharedString::from("text"),
                    value: slint::SharedString::default(),
                });
            }
        }
    });

    let dialog_weak4 = dialog.as_weak();
    dialog.on_field_selected(move |idx, name| {
        if let Some(d) = dialog_weak4.upgrade() {
            let entries = d.get_entries();
            let model = entries
                .as_any()
                .downcast_ref::<slint::VecModel<FieldEntry>>()
                .expect("entries must be VecModel<FieldEntry>");

            // Look up the field-type for the selected field name
            let available = d.get_available_fields();
            let field_type = (0..available.row_count())
                .filter_map(|i| available.row_data(i))
                .find(|fd| fd.name == name)
                .map(|fd| fd.field_type)
                .unwrap_or_else(|| slint::SharedString::from("text"));

            if let Some(mut row) = model.row_data(idx as usize) {
                row.field_name = name;
                row.field_type = field_type;
                model.set_row_data(idx as usize, row);
            }
        }
    });

    let dialog_weak5 = dialog.as_weak();
    dialog.on_delete_requested(move |idx| {
        if let Some(d) = dialog_weak5.upgrade() {
            let entries = d.get_entries();
            let model = entries
                .as_any()
                .downcast_ref::<slint::VecModel<FieldEntry>>()
                .expect("entries must be VecModel<FieldEntry>");
            if (idx as usize) < model.row_count() {
                model.remove(idx as usize);
            }
        }
    });

    // Open a nested ContentDialog when the user requests to add a content
    let dialog_weak6 = dialog.as_weak();
    dialog.on_add_content_requested(move || {
        let _ = open_content_dialog_for_book(dialog_weak6.clone());
    });

    let dialog_weak7 = dialog.as_weak();
    dialog.on_edit_content_requested(move |_idx| {
        if let Some(_d) = dialog_weak7.upgrade() {
            // TODO: open ContentDialog pre-filled with content at index _idx
        }
    });

    let dialog_weak8 = dialog.as_weak();
    dialog.on_delete_content_requested(move |idx| {
        if let Some(d) = dialog_weak8.upgrade() {
            let contents = d.get_added_contents();
            let model = contents
                .as_any()
                .downcast_ref::<slint::VecModel<ContentSummary>>()
                .expect("added_contents must be VecModel<ContentSummary>");
            if (idx as usize) < model.row_count() {
                model.remove(idx as usize);
            }
        }
    });

    dialog.show()?;
    Ok(())
}
