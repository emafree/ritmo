use slint::ComponentHandle;
use crate::{PeopleDialog, Suggestion, FieldEntry, FieldDefinition};
use super::role_dialog::open_role_dialog;

pub fn open_people_dialog() -> anyhow::Result<()> {
    let dialog = PeopleDialog::new()?;

    dialog.set_name_suggestions(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<Suggestion>::from(vec![])),
    ));
    dialog.set_role_suggestions(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<Suggestion>::from(vec![])),
    ));
    dialog.set_available_field_names(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<slint::SharedString>::from(vec![])),
    ));
    dialog.set_available_fields(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<FieldDefinition>::from(vec![])),
    ));
    dialog.set_optional_entries(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<FieldEntry>::from(vec![])),
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
            // TODO: collect name/role/entries and persist person to DB
            d.hide().unwrap();
        }
    });

    let dialog_weak3 = dialog.as_weak();
    dialog.on_name_text_changed(move |_text| {
        if let Some(d) = dialog_weak3.upgrade() {
            // TODO: filter name suggestions from DB based on _text
            d.set_name_suggestions(slint::ModelRc::from(
                std::rc::Rc::new(slint::VecModel::<Suggestion>::from(vec![])),
            ));
        }
    });

    dialog.on_name_selected(move |_id| {
        // TODO: set name-text with the selected suggestion's display-name
    });

    dialog.on_name_create_requested(move |_text| {
        // TODO: accept _text as a new person name
    });

    let dialog_weak4 = dialog.as_weak();
    dialog.on_role_text_changed(move |_text| {
        if let Some(d) = dialog_weak4.upgrade() {
            // TODO: filter role suggestions from DB based on _text
            d.set_role_suggestions(slint::ModelRc::from(
                std::rc::Rc::new(slint::VecModel::<Suggestion>::from(vec![])),
            ));
        }
    });

    dialog.on_role_selected(move |_id| {
        // TODO: set role-text with the selected suggestion's display-name
    });

    // When the user requests creating a new role, open RoleDialog
    dialog.on_role_create_requested(move |_text| {
        let _ = open_role_dialog();
    });

    dialog.on_optional_value_changed(move |_idx, _entry| {
        // TODO: update the optional-entries model at index _idx
    });

    dialog.on_optional_field_selected(move |_idx, _name| {
        // TODO: update field-type for the optional row at index _idx
    });

    dialog.on_optional_delete_requested(move |_idx| {
        // TODO: remove the optional row at index _idx from optional-entries model
    });

    dialog.show()?;
    Ok(())
}
