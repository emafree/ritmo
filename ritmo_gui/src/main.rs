slint::include_modules!();

use slint::SharedString;
use std::collections::HashMap;
use std::fs;

// Funzione di utilità per caricare le traduzioni da JSON
fn load_translations(lang: &str) -> HashMap<String, String> {
    let file = format!("ritmo_gui/i18n/{}.json", lang);
    let json = fs::read_to_string(&file).expect("Traduzioni non trovate!");
    serde_json::from_str(&json).expect("Errore parsing JSON!")
}

fn main() -> Result<(), slint::PlatformError> {
    // 1. Carica le traduzioni (scegli la lingua dinamicamente se vuoi)
    let translations = load_translations("it"); // o "en", ecc.

    // 2. Istanzia la finestra principale (auto-generata da main.slint)
    let main_window = MainWindow::new()?;

    // 3. Imposta le proprietà globali delle traduzioni
    let tr = Tr::global();
    let tr = window.global::<Tr>();
    for (k, v) in translations {
        tr.set_property(&k, SharedString::from(v));
    }

    // 4. Mostra la finestra e lancia l’event loop
    main_window.run()
}
