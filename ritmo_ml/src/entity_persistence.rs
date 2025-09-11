use sqlx::{Row, SqlitePool, Sqlite, Transaction, Executor};
use crate::entity_learner::MLEntityLearner;
use ritmo_errors::RitmoErr;

/// Salva i dati ML su DB
pub async fn save_ml_to_db(
    tx: &mut Transaction<'_, Sqlite>,
    ml: &MLEntityLearner,
    prefix: &str,
) -> Result<(), RitmoErr> {
    save_data(&mut **tx, &format!("{}_clusters", prefix), &ml.clusters).await?;
    save_data(&mut **tx, &format!("{}_learned_patterns", prefix), &ml.learned_patterns).await?;
    save_data(&mut **tx, &format!("{}_pattern_frequency", prefix), &ml.pattern_frequency).await?;
    let config = serde_json::json!({
        "minimum_confidence": ml.minimum_confidence,
        "minimum_frequency": ml.minimum_frequency
    });
    save_data(&mut **tx, &format!("{}_ml_config", prefix), &config).await?;
    Ok(())
}

// Funzione helper generica su qualunque Executor mutabile
async fn save_data<'e, E, T>(
    executor: E,
    data_type: &str,
    data_value: &T,
) -> Result<(), RitmoErr>
where
    E: Executor<'e, Database = Sqlite>,
    T: serde::Serialize,
{
    let json_string = serde_json::to_string(data_value)?;
    
    sqlx::query(
        "INSERT OR REPLACE INTO ml_data (data_type, data_json) VALUES (?, ?)",
    )
    .bind(data_type)
    .bind(json_string)
    .execute(executor)
    .await?;
    
    Ok(())
}

pub async fn save_scalar_to_db<T: serde::Serialize>(
    tx: &mut Transaction<'_, Sqlite>,
    data_type: &str,
    scalar_value: &T,
) -> Result<(), RitmoErr> {
    save_data(&mut **tx, data_type, scalar_value).await
}

/// Carica i dati ML dal DB
pub async fn load_ml_from_db(
    pool: &SqlitePool,
    prefix: &str,
) -> Result<MLEntityLearner, RitmoErr> {
    let clusters = load_data(pool, &format!("{}_clusters", prefix)).await?.unwrap_or_default();
    let learned_patterns = load_data(pool, &format!("{}_learned_patterns", prefix)).await?.unwrap_or_default();
    let pattern_frequency = load_data(pool, &format!("{}_pattern_frequency", prefix)).await?.unwrap_or_default();
    let config_json = load_data::<serde_json::Value>(pool, &format!("{}_ml_config", prefix))
        .await?
        .unwrap_or_else(|| serde_json::json!({
            "minimum_confidence": 0.85,
            "minimum_frequency": 3
        }));

    let minimum_confidence = config_json["minimum_confidence"].as_f64().unwrap_or(0.85);
    let minimum_frequency = config_json["minimum_frequency"].as_u64().map(|v| v as usize).unwrap_or(3);

    Ok(MLEntityLearner {
        clusters,
        learned_patterns,
        pattern_frequency,
        minimum_confidence,
        minimum_frequency,
    })
}

pub async fn load_scalar_from_db<T: for<'de> serde::Deserialize<'de>>(
    pool: &SqlitePool,
    data_type: &str,
) -> Result<Option<T>, RitmoErr> {
    load_data(pool, data_type).await
}

async fn load_data<T: for<'de> serde::Deserialize<'de>>(
    pool: &SqlitePool,
    data_type: &str,
) -> Result<Option<T>, RitmoErr> {
    let row = sqlx::query("SELECT data_json FROM ml_data WHERE data_type = ?")
        .bind(data_type)
        .fetch_optional(pool)
        .await?;
    match row {
        Some(r) => {
            let json_string: String = r.try_get("data_json")?;
            let data: T = serde_json::from_str(&json_string)?;
            Ok(Some(data))
        }
        None => Ok(None),
    }
}