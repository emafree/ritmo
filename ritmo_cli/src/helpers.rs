//! Helper functions used across commands

use ritmo_config::{detect_portable_library, AppSettings};
use std::path::PathBuf;

/// Helper: determina il path della libreria da usare
pub fn get_library_path(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(path) = cli_library {
        Ok(path.clone())
    } else if let Some(portable) = detect_portable_library() {
        Ok(portable)
    } else if let Some(path) = &app_settings.last_library_path {
        Ok(path.clone())
    } else {
        Err("Nessuna libreria configurata. Usa 'ritmo init' per inizializzare una libreria".into())
    }
}

/// Helper: converte data YYYY-MM-DD in timestamp UNIX
pub fn parse_date_to_timestamp(date_str: &str) -> Result<i64, Box<dyn std::error::Error>> {
    use chrono::NaiveDate;

    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| format!("Formato data non valido: '{}'. Usa YYYY-MM-DD", date_str))?;

    // Converte a timestamp UNIX (inizio del giorno in UTC)
    Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
}

/// Helper: calcola timestamp di N giorni fa
pub fn timestamp_days_ago(days: i64) -> i64 {
    use chrono::{Duration, Utc};

    let now = Utc::now();
    let past = now - Duration::days(days);
    past.timestamp()
}

/// Helper: calcola timestamp di N mesi fa (approssimato a 30 giorni per mese)
pub fn timestamp_months_ago(months: i64) -> i64 {
    use chrono::{Duration, Utc};

    let now = Utc::now();
    let past = now - Duration::days(months * 30);
    past.timestamp()
}
