use crate::db::DbPool;
use crate::utils::parse_uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::mysql::MySqlRow;
use sqlx::Row;
use unified_shared::error::DomainError;
use uuid::Uuid;

const INTERNAL_ERR: &str = "internal error";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFamily {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub model_type: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewModelFamily {
    pub project_id: Uuid,
    pub name: String,
    pub model_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelImplementation {
    pub id: Uuid,
    pub project_id: Uuid,
    pub family_id: Uuid,
    pub name: String,
    pub repo_url: Option<String>,
    pub repo_reference: Option<String>,
    pub runtime_type: String,
    pub config_path: Option<String>,
    pub default_task_types: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewModelImplementation {
    pub project_id: Uuid,
    pub family_id: Uuid,
    pub name: String,
    pub repo_url: Option<String>,
    pub repo_reference: Option<String>,
    pub runtime_type: String,
    pub config_path: Option<String>,
    pub default_task_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: Uuid,
    pub project_id: Uuid,
    pub model_impl_id: Uuid,
    pub name: String,
    pub weights_uri: Option<String>,
    pub step: Option<i64>,
    pub training_summary: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewCheckpoint {
    pub project_id: Uuid,
    pub model_impl_id: Uuid,
    pub name: String,
    pub weights_uri: Option<String>,
    pub step: Option<i64>,
    pub training_summary: Option<Value>,
}

fn row_to_family(row: &MySqlRow) -> Result<ModelFamily, DomainError> {
    Ok(ModelFamily {
        id: parse_uuid(row.try_get::<String, _>("id")?.as_str())?,
        project_id: parse_uuid(row.try_get::<String, _>("project_id")?.as_str())?,
        name: row.try_get("name")?,
        model_type: row.try_get("model_type")?,
        description: row.try_get::<Option<String>, _>("description")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_impl(row: &MySqlRow) -> Result<ModelImplementation, DomainError> {
    let default_task_types = row
        .try_get::<Option<String>, _>("default_task_types")?
        .map(|raw| serde_json::from_str(&raw).unwrap_or_default())
        .unwrap_or_default();

    Ok(ModelImplementation {
        id: parse_uuid(row.try_get::<String, _>("id")?.as_str())?,
        project_id: parse_uuid(row.try_get::<String, _>("project_id")?.as_str())?,
        family_id: parse_uuid(row.try_get::<String, _>("family_id")?.as_str())?,
        name: row.try_get("name")?,
        repo_url: row.try_get::<Option<String>, _>("repo_url")?,
        repo_reference: row.try_get::<Option<String>, _>("repo_reference")?,
        runtime_type: row.try_get("runtime_type")?,
        config_path: row.try_get::<Option<String>, _>("config_path")?,
        default_task_types,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_checkpoint(row: &MySqlRow) -> Result<Checkpoint, DomainError> {
    let summary = row
        .try_get::<Option<String>, _>("training_summary")?
        .map(|raw| serde_json::from_str(&raw).unwrap_or(Value::Null));

    Ok(Checkpoint {
        id: parse_uuid(row.try_get::<String, _>("id")?.as_str())?,
        project_id: parse_uuid(row.try_get::<String, _>("project_id")?.as_str())?,
        model_impl_id: parse_uuid(row.try_get::<String, _>("model_impl_id")?.as_str())?,
        name: row.try_get("name")?,
        weights_uri: row.try_get("weights_uri")?,
        step: row.try_get("step")?,
        training_summary: summary,
        created_at: row.try_get("created_at")?,
    })
}

pub async fn list_families(pool: &DbPool, project_id: &Uuid) -> Result<Vec<ModelFamily>, DomainError> {
    let rows = sqlx::query(
        "SELECT id, project_id, name, model_type, description, created_at, updated_at FROM model_families WHERE project_id = ? ORDER BY created_at DESC",
    )
    .bind(project_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(|e| DomainError::Internal(e.to_string()))?;

    rows.iter().map(row_to_family).collect()
}

pub async fn create_family(pool: &DbPool, payload: NewModelFamily) -> Result<ModelFamily, DomainError> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    sqlx::query("INSERT INTO model_families (id, project_id, name, model_type, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(id.to_string())
        .bind(payload.project_id.to_string())
        .bind(&payload.name)
        .bind(&payload.model_type)
        .bind(&payload.description)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    Ok(ModelFamily {
        id,
        project_id: payload.project_id,
        name: payload.name,
        model_type: payload.model_type,
        description: payload.description,
        created_at: now,
        updated_at: now,
    })
}

pub async fn list_impls(pool: &DbPool, project_id: &Uuid) -> Result<Vec<ModelImplementation>, DomainError> {
    let rows = sqlx::query(
        "SELECT id, project_id, family_id, name, repo_url, repo_reference, runtime_type, config_path, default_task_types, created_at, updated_at FROM model_impls WHERE project_id = ? ORDER BY created_at DESC",
    )
    .bind(project_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(|e| DomainError::Internal(e.to_string()))?;

    rows.iter().map(row_to_impl).collect()
}

pub async fn create_impl(pool: &DbPool, payload: NewModelImplementation) -> Result<ModelImplementation, DomainError> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let default_task_types = serde_json::to_string(&payload.default_task_types)
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    sqlx::query("INSERT INTO model_impls (id, project_id, family_id, name, repo_url, repo_reference, runtime_type, config_path, default_task_types, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(id.to_string())
        .bind(payload.project_id.to_string())
        .bind(payload.family_id.to_string())
        .bind(&payload.name)
        .bind(&payload.repo_url)
        .bind(&payload.repo_reference)
        .bind(&payload.runtime_type)
        .bind(&payload.config_path)
        .bind(default_task_types)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    Ok(ModelImplementation {
        id,
        project_id: payload.project_id,
        family_id: payload.family_id,
        name: payload.name,
        repo_url: payload.repo_url,
        repo_reference: payload.repo_reference,
        runtime_type: payload.runtime_type,
        config_path: payload.config_path,
        default_task_types: payload.default_task_types,
        created_at: now,
        updated_at: now,
    })
}

pub async fn list_checkpoints(pool: &DbPool, model_impl_id: &Uuid) -> Result<Vec<Checkpoint>, DomainError> {
    let rows = sqlx::query(
        "SELECT id, project_id, model_impl_id, name, weights_uri, step, training_summary, created_at FROM checkpoints WHERE model_impl_id = ? ORDER BY created_at DESC",
    )
    .bind(model_impl_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(|e| DomainError::Internal(e.to_string()))?;

    rows.iter().map(row_to_checkpoint).collect()
}

pub async fn create_checkpoint(pool: &DbPool, payload: NewCheckpoint) -> Result<Checkpoint, DomainError> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let summary_str = match payload.training_summary {
        Some(ref value) => Some(
            serde_json::to_string(value).map_err(|e| DomainError::Internal(e.to_string()))?,
        ),
        None => None,
    };

    sqlx::query("INSERT INTO checkpoints (id, project_id, model_impl_id, name, weights_uri, step, training_summary, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(id.to_string())
        .bind(payload.project_id.to_string())
        .bind(payload.model_impl_id.to_string())
        .bind(&payload.name)
        .bind(&payload.weights_uri)
        .bind(payload.step)
        .bind(summary_str)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    Ok(Checkpoint {
        id,
        project_id: payload.project_id,
        model_impl_id: payload.model_impl_id,
        name: payload.name,
        weights_uri: payload.weights_uri,
        step: payload.step,
        training_summary: payload.training_summary,
        created_at: now,
    })
}

