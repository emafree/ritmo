use std::path::Path;
use std::fs;

/// Crea la struttura di directory di storage (media, config, bootstrap, ecc.)
pub fn create_storage_dirs<P: AsRef<Path>>(root: P) -> Result<(), std::io::Error> {
    let root = root.as_ref();
    fs::create_dir_all(root.join("storage"))?;
    fs::create_dir_all(root.join("config"))?;
    fs::create_dir_all(root.join("bootstrap"))?;
    fs::create_dir_all(root.join("bootstrap/portable_app"))?;
    Ok(())
}