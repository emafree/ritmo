use std::env;
use std::path::PathBuf;

/// Rileva se il programma è in esecuzione da bootstrap/portable_app/
/// Ritorna Some(library_root) se è in modalità portabile, None altrimenti
pub fn detect_portable_library() -> Option<PathBuf> {
    let exe_path = env::current_exe().ok()?;

    // Controlla se il path dell'eseguibile contiene "bootstrap/portable_app"
    let exe_dir = exe_path.parent()?;

    // Verifica pattern: .../bootstrap/portable_app/ritmo_gui
    if exe_dir.ends_with("portable_app") {
        if let Some(bootstrap_dir) = exe_dir.parent() {
            if bootstrap_dir.ends_with("bootstrap") {
                // La root della libreria è due livelli sopra
                return bootstrap_dir.parent().map(|p| p.to_path_buf());
            }
        }
    }

    None
}

/// Controlla se il programma è in esecuzione in modalità portabile
pub fn is_running_portable() -> bool {
    detect_portable_library().is_some()
}

/// Verifica se un path è una libreria Ritmo valida
/// (controlla l'esistenza delle directory essenziali)
#[cfg(test)]
pub fn is_valid_library(path: &std::path::Path) -> bool {
    path.is_dir()
        && path.join("database").is_dir()
        && path.join("storage").is_dir()
        && path.join("config").is_dir()
        && path.join("bootstrap").is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_valid_library() {
        let temp = TempDir::new().unwrap();
        let lib_path = temp.path();

        // Non valida inizialmente
        assert!(!is_valid_library(lib_path));

        // Crea struttura
        fs::create_dir_all(lib_path.join("database")).unwrap();
        fs::create_dir_all(lib_path.join("storage")).unwrap();
        fs::create_dir_all(lib_path.join("config")).unwrap();
        fs::create_dir_all(lib_path.join("bootstrap")).unwrap();

        // Ora è valida
        assert!(is_valid_library(lib_path));
    }
}
