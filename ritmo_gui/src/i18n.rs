// src/i18n.rs
use crate::{MainWindow, Tr, Translations};
use rust_embed::RustEmbed;
use slint::ComponentHandle;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct I18nFiles;

pub fn load_translations(lang: &str) -> Translations {
    let json = I18nFiles::get(&format!("{}.json", lang))
        .or_else(|| I18nFiles::get("en.json"))
        .expect("en.json not found");

    let content =
        std::str::from_utf8(json.data.as_ref()).expect("invalid utf8 in translation file");

    let map: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(content).expect("invalid JSON");

    let get = |key: &str| -> slint::SharedString {
        map.get(key).and_then(|v| v.as_str()).unwrap_or(key).into()
    };

    Translations {
        accept: get("accept"),
        all_books: get("all-books"),
        app_name: get("app-name"),
        books: get("books"),
        cancel: get("cancel"),
        completed: get("completed"),
        contents: get("contents"),
        delete: get("delete"),
        disable: get("disable"),
        edit: get("edit"),
        enable: get("enable"),
        favorites: get("favorites"),
        field_author: get("field-author"),
        field_language: get("field-language"),
        field_name: get("field-name"),
        field_role: get("field-role"),
        field_title: get("field-title"),
        grid: get("grid"),
        library: get("library"),
        list: get("list"),
        new_book: get("new-book"),
        new_content: get("new-content"),
        new_person: get("new-person"),
        new_role: get("new-role"),
        quit: get("quit"),
        reading: get("reading"),
        save: get("save"),
        select: get("select"),
        select_all: get("select-all"),
        select_none: get("select-none"),
        field_original_title: get("field-original-title"),
        field_publication_date: get("field-publication-date"),
        field_isbn: get("field-isbn"),
        field_pages: get("field-pages"),
        field_notes: get("field-notes"),
        field_people: get("field-people"),
    }
}

pub fn apply_translations(win: &MainWindow, lang: &str) {
    let tr = win.global::<Tr>();
    tr.set_t(load_translations(lang));
}
