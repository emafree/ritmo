use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Directory di configurazione non trovata")]
    ConfigDirNotFound,

    #[error("Errore I/O: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Errore parsing TOML: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Errore serializzazione TOML: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    #[error("Path non valido: {0}")]
    InvalidPath(String),
}
