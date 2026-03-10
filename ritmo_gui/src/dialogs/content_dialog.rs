// src/dialogs/content_dialog.rs
use crate::{ContentDialog, FieldDefinition, FieldEntry, Suggestion, Tr};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::rc::Rc;

pub fn content_optional_fields(tr: &crate::Translations) -> Vec<FieldDefinition> {
    vec![
        FieldDefinition {
            name: tr.field_title.clone(),
            data_kind: "string".into(),
            enum_values: ModelRc::default(),
        },
        FieldDefinition {
            name: tr.field_author.clone(),
            data_kind: "person".into(),
            enum_values: ModelRc::default(),
        },
        FieldDefinition {
            name: tr.field_language.clone(),
            data_kind: "enum".into(),
            enum_values: ModelRc::default(), // TODO: popolare con lingue reali
        },
        FieldDefinition {
            name: tr.field_people.clone(),
            data_kind: "person".into(),
            enum_values: ModelRc::default(),
        },
    ]
}

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

fn maybe_push_ghost(model: &VecModel<FieldEntry>, idx: usize, row: &FieldEntry) {
    if is_row_complete(row) && idx == model.row_count() - 1 {
        model.push(empty_placeholder());
    }
}

fn init_dialog(dialog: &ContentDialog, fields: &[FieldDefinition]) {
    let field_names: Vec<slint::SharedString> =
        fields.iter().map(|f| f.name.clone()).collect();

    dialog.set_available_field_names(ModelRc::from(Rc::new(
        VecModel::<slint::SharedString>::from(field_names),
    )));
    dialog.set_available_fields(ModelRc::from(Rc::new(
        VecModel::<FieldDefinition>::from(fields.to_vec()),
    )));
    dialog.set_entries(ModelRc::from(Rc::new(
        VecModel::<FieldEntry>::from(vec![first_placeholder(fields)]),
    )));
    dialog.set_name_suggestions(ModelRc::from(Rc::new(
        VecModel::<Suggestion>::from(vec![]),
    )));
    dialog.set_role_suggestions(ModelRc::from(Rc::new(
        VecModel::<Suggestion>::from(vec![]),
    )));
}

fn wire_callbacks(dialog: &ContentDialog) {
    let dw = dialog.as_weak();
    dialog.on_value_changed(move |idx, entry| {
        if let Some(d) = dw.upgrade() {
            let entries = d.get_entries();
            let model = downcast_entries(&entries);
            model.set_row_data(idx as usize, entry.clone());
            maybe_push_ghost(model, idx as usize, &entry);
        }
    });

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
            // TODO: ML suggestions
        }
    });

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
    dialog.on_name_create_requested(move |_idx, _text| {});

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
            // TODO: ML suggestions
        }
    });

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
    dialog.on_role_create_requested(move |_idx, _text| {});
}

pub fn open_content_dialog(win: &crate::MainWindow) -> anyhow::Result<()> {
    let dialog = ContentDialog::new()?;
    let tr = win.global::<Tr>().get_t();
    let fields = content_optional_fields(&tr);

    init_dialog(&dialog, &fields);

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

    wire_callbacks(&dialog);
    dialog.show()?;
    Ok(())
}

/// Apre ContentDialog nel contesto di un BookDialog.
/// Riceve i fields già costruiti dal chiamante per evitare dipendenza da MainWindow.
pub fn open_content_dialog_for_book(
    book_dialog_weak: slint::Weak<crate::BookDialog>,
    fields: Vec<FieldDefinition>,
) -> anyhow::Result<()> {
    let dialog = ContentDialog::new()?;
    init_dialog(&dialog, &fields);

    let dw = dialog.as_weak();
    dialog.on_cancelled(move || { dw.upgrade().map(|d| d.hide().unwrap()); });

    let dw = dialog.as_weak();
    dialog.on_accepted(move || {
        if let Some(d) = dw.upgrade() {
            if let Some(book) = book_dialog_weak.upgrade() {
                let entries = d.get_entries();
                let title = (0..entries.row_count())
                    .filter_map(|i| entries.row_data(i))
                    .find(|e| e.data_kind == "string" && !e.value.is_empty())
                    .map(|e| e.value)
                    .unwrap_or_default();
                let author = (0..entries.row_count())
                    .filter_map(|i| entries.row_data(i))
                    .find(|e| e.data_kind == "person" && !e.person_name.is_empty())
                    .map(|e| e.person_name)
                    .unwrap_or_default();
                let new_content = crate::ContentSummary { title, author };
                let current = book.get_added_contents();
                let mut items: Vec<crate::ContentSummary> =
                    (0..current.row_count()).filter_map(|i| current.row_data(i)).collect();
                items.push(new_content);
                book.set_added_contents(ModelRc::from(Rc::new(VecModel::from(items))));
            }
            d.hide().unwrap();
        }
    });

    wire_callbacks(&dialog);
    dialog.show()?;
    Ok(())
}
