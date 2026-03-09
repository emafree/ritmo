use slint::ComponentHandle;
use crate::{RoleDialog, Suggestion};

pub fn open_role_dialog() -> anyhow::Result<()> {
    let dialog = RoleDialog::new()?;

    dialog.set_role_suggestions(slint::ModelRc::from(
        std::rc::Rc::new(slint::VecModel::<Suggestion>::from(vec![])),
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
            // TODO: persist the new role
            d.hide().unwrap();
        }
    });

    let dialog_weak3 = dialog.as_weak();
    dialog.on_role_text_changed(move |_text| {
        if let Some(_d) = dialog_weak3.upgrade() {
            // TODO: filter role suggestions from DB
        }
    });

    dialog.on_role_selected(move |_id| {
        // TODO: set role-text with selected name
    });

    dialog.on_role_create_requested(move |_text| {
        // TODO: accept text as new role name
    });

    dialog.show()?;
    Ok(())
}
