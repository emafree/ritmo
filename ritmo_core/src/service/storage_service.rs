use ritmo_db_core::LibraryConfig;
use ritmo_errors::{RitmoErr, RitmoResult};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub fn save_epub_file(book: &mut Book, bytes: &[u8], config: &LibraryConfig) -> RitmoResult<()> {
    // Genera percorso e hash se non già presenti
    book.set_book_persistence();
    let file_link = book
        .file_link
        .as_ref()
        .ok_or_else(|| RitmoErr::Generic("file_link non impostato".into()))?;

    // Usa lo storage_path dalla configurazione!
    let base_dir = config.canonical_storage_path(); // <- dinamico!
    let relative_path = Path::new(file_link);
    let full_path = base_dir.join(relative_path);

    // Crea directory se non esistono
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).map_err(RitmoErr::FileAccessError)?;
    }

    // Scrivi il file
    let mut file = File::create(&full_path).map_err(RitmoErr::FileAccessError)?;
    file.write_all(bytes).map_err(RitmoErr::FileAccessError)?;
    file.sync_all().map_err(RitmoErr::FileAccessError)?;

    // Aggiorna la dimensione file
    let metadata = file.metadata().map_err(RitmoErr::FileAccessError)?;
    book.file_size = Some(metadata.len() as usize);

    Ok(())
}
