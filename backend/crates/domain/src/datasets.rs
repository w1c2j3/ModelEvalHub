use crate::db::DbPool;
use crate::utils::parse_uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::mysql::MySqlRow;
use sqlx::Row;
use unified_shared::error::DomainError;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub version: Option<String>,
    pub storage_uri: Option<String>,
    pub schema: Option<Value>,
    pub num_samples: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewDataset {
    pub project_id: Uuid,
    pub name: String,
    pub version: Option<String>,
    pub storage_uri: Option<String>,
    pub schema: Option<Value>,
    pub num_samples: Option<i64>,
}

fn row_to_dataset(row: &MySqlRow) -> Result<Dataset, DomainError> {
    let schema = row
        .try_get::<Option<String>, _>("schema_json")?
        .map(|raw| serde_json::from_str(&raw).unwrap_or(Value::Null));
    Ok(Dataset {
        id: parse_uuid(row.try_get::<String, _>("id")?.as_str())?,
        project_id: parse_uuid(row.try_get::<String, _>("project_id")?.as_str())?,
        name: row.try_get("name")?,
        version: row.try_get("version")?,
        storage_uri: row.try_get("storage_uri")?,
        schema,
        num_samples: row.try_get("num_samples")?,
        created_at: row.try_get("created_at")?,
    })
}

pub async fn list(pool: &DbPool, project_id: &Uuid) -> Result<Vec<Dataset>, DomainError> {
    let rows = sqlx::query(
        "SELECT id, project_id, name, version, storage_uri, schema_json, num_samples, created_at FROM datasets WHERE project_id = ? ORDER BY created_at DESC",
    )
    .bind(project_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(|e| DomainError::Internal(e.to_string()))?;

    rows.iter().map(row_to_dataset).collect()
}

pub async fn create(pool: &DbPool, payload: NewDataset) -> Result<Dataset, DomainError> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let schema_str = match payload.schema {
        Some(ref value) => {
            Some(serde_json::to_string(value).map_err(|e| DomainError::Internal(e.to_string()))?)
        }
        None => None,
    };

    sqlx::query("INSERT INTO datasets (id, project_id, name, version, storage_uri, schema_json, num_samples, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(id.to_string())
        .bind(payload.project_id.to_string())
        .bind(&payload.name)
        .bind(&payload.version)
        .bind(&payload.storage_uri)
        .bind(schema_str)
        .bind(payload.num_samples)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    Ok(Dataset {
        id,
        project_id: payload.project_id,
        name: payload.name,
        version: payload.version,
        storage_uri: payload.storage_uri,
        schema: payload.schema,
        num_samples: payload.num_samples,
        created_at: now,
    })
}
