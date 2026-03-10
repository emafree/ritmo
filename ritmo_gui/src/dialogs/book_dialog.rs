// src/dialogs/book_dialog.rs
use super::content_dialog::open_content_dialog_for_book;
use crate::{BookDialog, ContentSummary, FieldDefinition, FieldEntry, Suggestion};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::rc::Rc;

fn first_placeholder(fields: &[FieldDefinition]) -> FieldEntry {
    match fields.first() {
        Some(f) => FieldEntry {
            field_name: f.name.clone(),
            data_kind: f.data_kind.clone(),
            value: slint::SharedString::default(),
            person_id: 0,
            person_name: slint::SharedString::default(),
            role_id: 0,
            role_name: slint::SharedString::default(),
        },
        None => empty_placeholder(),
    }
}

fn empty_placeholder() -> FieldEntry {
    FieldEntry {
        field_name: slint::SharedString::default(),
        data_kind: slint::SharedString::from("string"),
        value: slint::SharedString::default(),
        person_id: 0,
        person_name: slint::SharedString::default(),
        role_id: 0,
        role_name: slint::SharedString::default(),
    }
}

fn is_row_complete(entry: &FieldEntry) -> bool {
    if entry.data_kind == "person" {
        !entry.person_name.is_empty() && !entry.role_name.is_empty()
    } else {
        !entry.value.is_empty()
    }
}

fn downcast_entries(model: &ModelRc<FieldEntry>) -> &VecModel<FieldEntry> {
    model.as_any().downcast_ref::<VecModel<FieldEntry>>()
        .expect("entries must be VecModel<FieldEntry>")
}

/// Aggiunge una ghost row vuota se la riga appena modificata è completa ed è l'ultima.
fn maybe_push_ghost(model: &VecModel<FieldEntry>, idx: usize, row: &FieldEntry) {
    if is_row_complete(row) && idx == model.row_count() - 1 {
        model.push(empty_placeholder());
    }
}

pub fn open_book_dialog(
    win: &crate::MainWindow,
    book_fields: Vec<crate::FieldDefinition>,
    content_fields: Vec<crate::FieldDefinition>,
) -> anyhow::Result<()> {
    let dialog = BookDialog::new()?;

    let fields = book_fields;
    let field_names: Vec<slint::SharedString> =
        fields.iter().map(|f| f.name.clone()).collect();

    dialog.set_available_field_names(ModelRc::from(Rc::new(
        VecModel::<slint::SharedString>::from(field_names),
    )));
    dialog.set_available_fields(ModelRc::from(Rc::new(
        VecModel::<FieldDefinition>::from(fields.clone()),
    )));
    dialog.set_entries(ModelRc::from(Rc::new(
        VecModel::<FieldEntry>::from(vec![first_placeholder(&fields)]),
    )));
    dialog.set_added_contents(ModelRc::from(Rc::new(
        VecModel::<ContentSummary>::from(vec![]),
    )));
    dialog.set_name_suggestions(ModelRc::from(Rc::new(
        VecModel::<Suggestion>::from(vec![]),
    )));
    dialog.set_role_suggestions(ModelRc::from(Rc::new(
        VecModel::<Suggestion>::from(vec![]),
    )));

    // ── Cancelled / Accepted ─────────────────────────────────────
    let dw = dialog.as_weak();
    dialog.on_cancelled(move || { dw.upgrade().map(|d| d.hide().unwrap()); });

    let dw = dialog.as_weak();
    dialog.on_accepted(move || {
        if let Some(d) = dw.upgrade() {
            let entries = d.get_entries();
            let _rows: Vec<FieldEntry> = (0..entries.row_count())
                .filter_map(|i| entries.row_data(i))
                .filter(|e| is_row_complete(e))
                .collect();
            // TODO: persist to DB
            d.hide().unwrap();
        }
    });

    // ── value-changed (righe non-person) ─────────────────────────
    let dw = dialog.as_weak();
    dialog.on_value_changed(move |idx, entry| {
        if let Some(d) = dw.upgrade() {
            let entries = d.get_entries();
            let model = downcast_entries(&entries);
            model.set_row_data(idx as usize, entry.clone());
            maybe_push_ghost(model, idx as usize, &entry);
        }
    });

    // ── field-selected ───────────────────────────────────────────
    let dw = dialog.as_weak();
    dialog.on_field_selected(move |idx, name| {
        if let Some(d) = dw.upgrade() {
            let entries = d.get_entries();
            let model = downcast_entries(&entries);
            let available = d.get_available_fields();
            let data_kind = (0..available.row_count())
                .filter_map(|i| available.row_data(i))
                .find(|fd| fd.name == name)
                .map(|fd| fd.data_kind)
                .unwrap_or_else(|| "string".into());
            if let Some(mut row) = model.row_data(idx as usize) {
                row.field_name = name;
                row.data_kind = data_kind;
                row.value = slint::SharedString::default();
                row.person_name = slint::SharedString::default();
                row.role_name = slint::SharedString::default();
                model.set_row_data(idx as usize, row);
            }
        }
    });

    // ── delete-requested ─────────────────────────────────────────
    let dw = dialog.as_weak();
    dialog.on_delete_requested(move |idx| {
        if let Some(d) = dw.upgrade() {
            let entries = d.get_entries();
            let model = downcast_entries(&entries);
            if (idx as usize) < model.row_count() {
                model.remove(idx as usize);
            }
        }
    });

    // ── People — name-text-changed ───────────────────────────────
    let dw = dialog.as_weak();
    dialog.on_name_text_changed(move |idx, text| {
        if let Some(d) = dw.upgrade() {
            let entries = d.get_entries();
            let model = downcast_entries(&entries);
            if let Some(mut row) = model.row_data(idx as usize) {
                row.person_name = text;
                model.set_row_data(idx as usize, row.clone());
                maybe_push_ghost(model, idx as usize, &row);
            }
            // TODO: interroga ML, aggiorna name-suggestions
        }
    });

    // ── People — name-selected ───────────────────────────────────
    let dw = dialog.as_weak();
    dialog.on_name_selected(move |idx, id| {
        if let Some(d) = dw.upgrade() {
            let entries = d.get_entries();
            let model = downcast_entries(&entries);
            let suggestions = d.get_name_suggestions();
            if let Some(s) = (0..suggestions.row_count())
                .filter_map(|i| suggestions.row_data(i))
                .find(|s| s.id == id)
            {
                if let Some(mut row) = model.row_data(idx as usize) {
                    row.person_id = id;
                    row.person_name = s.display_name;
                    model.set_row_data(idx as usize, row.clone());
                    maybe_push_ghost(model, idx as usize, &row);
                }
            }
        }
    });

    let dw = dialog.as_weak();
    dialog.on_name_create_requested(move |_idx, _text| {
        // Nome nuovo — verrà creato nel DB al salvataggio
    });

    // ── People — role-text-changed ───────────────────────────────
    let dw = dialog.as_weak();
    dialog.on_role_text_changed(move |idx, text| {
        if let Some(d) = dw.upgrade() {
            let entries = d.get_entries();
            let model = downcast_entries(&entries);
            if let Some(mut row) = model.row_data(idx as usize) {
                row.role_name = text;
                model.set_row_data(idx as usize, row.clone());
                maybe_push_ghost(model, idx as usize, &row);
            }
            // TODO: interroga ML, aggiorna role-suggestions
        }
    });

    // ── People — role-selected ───────────────────────────────────
    let dw = dialog.as_weak();
    dialog.on_role_selected(move |idx, id| {
        if let Some(d) = dw.upgrade() {
            let entries = d.get_entries();
            let model = downcast_entries(&entries);
            let suggestions = d.get_role_suggestions();
            if let Some(s) = (0..suggestions.row_count())
                .filter_map(|i| suggestions.row_data(i))
                .find(|s| s.id == id)
            {
                if let Some(mut row) = model.row_data(idx as usize) {
                    row.role_id = id;
                    row.role_name = s.display_name;
                    model.set_row_data(idx as usize, row.clone());
                    maybe_push_ghost(model, idx as usize, &row);
                }
            }
        }
    });

    let dw = dialog.as_weak();
    dialog.on_role_create_requested(move |_idx, _text| {
        // Ruolo nuovo — verrà creato nel DB al salvataggio
    });

    // ── Contents ─────────────────────────────────────────────────
    let dw = dialog.as_weak();
    dialog.on_add_content_requested(move || {
        let _ = open_content_dialog_for_book(dw.clone(), content_fields.clone());
    });

    let dw = dialog.as_weak();
    dialog.on_edit_content_requested(move |_idx| {
        // TODO: open ContentDialog pre-filled
    });

    let dw = dialog.as_weak();
    dialog.on_delete_content_requested(move |idx| {
        if let Some(d) = dw.upgrade() {
            let contents = d.get_added_contents();
            let model = contents.as_any().downcast_ref::<VecModel<ContentSummary>>()
                .expect("added_contents must be VecModel<ContentSummary>");
            if (idx as usize) < model.row_count() {
                model.remove(idx as usize);
            }
        }
    });

    dialog.show()?;
    Ok(())
}
