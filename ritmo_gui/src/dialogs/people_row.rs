// src/dialogs/people_row.rs
// Gestione delle righe PeopleRow — riutilizzabile da book_dialog e content_dialog.
// Ogni riga ha nome+ruolo con autocomplete ML.

use crate::{PersonEntry, Suggestion};
use slint::{Model, ModelRc, VecModel};
use std::rc::Rc;

/// Stato interno di una singola riga people, gestito dal Rust.
/// Il Rust mantiene un Vec<PeopleRowState> parallelo al VecModel<PersonEntry> in Slint.
pub struct PeopleRowState {
    pub entry: PersonEntry,
}

/// Crea il VecModel iniziale con una sola ghost row vuota.
pub fn initial_people_model() -> ModelRc<PersonEntry> {
    ModelRc::from(Rc::new(VecModel::<PersonEntry>::from(vec![
        ghost_row(),
    ])))
}

/// Ghost row vuota — placeholder per aggiungere una nuova persona.
pub fn ghost_row() -> PersonEntry {
    PersonEntry {
        person_id: 0,
        person_name: slint::SharedString::default(),
        role_id: 0,
        role_name: slint::SharedString::default(),
    }
}

/// Aggiorna il nome di una riga e, se è l'ultima, aggiunge una nuova ghost row.
pub fn on_name_changed(
    model: &VecModel<PersonEntry>,
    idx: usize,
    text: slint::SharedString,
) {
    if let Some(mut row) = model.row_data(idx) {
        row.person_name = text;
        model.set_row_data(idx, row.clone());

        // Se la riga è compilata (nome non vuoto) ed è l'ultima, aggiungi ghost row
        if !row.person_name.is_empty()
            && !row.role_name.is_empty()
            && idx == model.row_count() - 1
        {
            model.push(ghost_row());
        }
    }
}

/// Aggiorna il ruolo di una riga e, se è l'ultima e completa, aggiunge ghost row.
pub fn on_role_changed(
    model: &VecModel<PersonEntry>,
    idx: usize,
    text: slint::SharedString,
) {
    if let Some(mut row) = model.row_data(idx) {
        row.role_name = text;
        model.set_row_data(idx, row.clone());

        if !row.person_name.is_empty()
            && !row.role_name.is_empty()
            && idx == model.row_count() - 1
        {
            model.push(ghost_row());
        }
    }
}

/// Imposta nome e id dopo che l'utente ha scelto un suggerimento.
pub fn on_name_selected(
    model: &VecModel<PersonEntry>,
    idx: usize,
    id: i32,
    suggestions: &[Suggestion],
) {
    if let Some(mut row) = model.row_data(idx) {
        if let Some(s) = suggestions.iter().find(|s| s.id == id) {
            row.person_id = id;
            row.person_name = s.display_name.clone();
            model.set_row_data(idx, row.clone());

            if !row.role_name.is_empty() && idx == model.row_count() - 1 {
                model.push(ghost_row());
            }
        }
    }
}

/// Imposta ruolo e id dopo che l'utente ha scelto un suggerimento.
pub fn on_role_selected(
    model: &VecModel<PersonEntry>,
    idx: usize,
    id: i32,
    suggestions: &[Suggestion],
) {
    if let Some(mut row) = model.row_data(idx) {
        if let Some(s) = suggestions.iter().find(|s| s.id == id) {
            row.role_id = id;
            row.role_name = s.display_name.clone();
            model.set_row_data(idx, row.clone());

            if !row.person_name.is_empty() && idx == model.row_count() - 1 {
                model.push(ghost_row());
            }
        }
    }
}

/// Rimuove una riga dal model (non rimuove la ghost row finale).
pub fn on_delete(model: &VecModel<PersonEntry>, idx: usize) {
    if idx < model.row_count().saturating_sub(1) {
        model.remove(idx);
    }
}

/// Raccoglie le righe complete (nome + ruolo entrambi non vuoti, esclusa ghost row).
pub fn collect_entries(model: &VecModel<PersonEntry>) -> Vec<PersonEntry> {
    (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter(|e| !e.person_name.is_empty() && !e.role_name.is_empty())
        .collect()
}
