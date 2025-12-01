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
pub struct Experiment {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub scenario_type: Option<String>,
    pub tasks: Vec<Uuid>,
    pub global_config: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewExperiment {
    pub project_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub scenario_type: Option<String>,
    pub tasks: Vec<Uuid>,
    pub global_config: Option<Value>,
}

fn row_to_experiment(row: &MySqlRow) -> Result<Experiment, DomainError> {
    let tasks_raw: String = row.try_get("tasks_json")?;
    let tasks_vec: Vec<String> =
        serde_json::from_str(&tasks_raw).map_err(|e| DomainError::Internal(e.to_string()))?;
    let tasks = tasks_vec
        .iter()
        .map(|id| parse_uuid(id.as_str()))
        .collect::<Result<Vec<_>, _>>()?;

    let global_config = row
        .try_get::<Option<String>, _>("global_config_json")?
        .map(|raw| serde_json::from_str(&raw).unwrap_or(Value::Null));

    Ok(Experiment {
        id: parse_uuid(row.try_get::<String, _>("id")?.as_str())?,
        project_id: parse_uuid(row.try_get::<String, _>("project_id")?.as_str())?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        scenario_type: row.try_get("scenario_type")?,
        tasks,
        global_config,
        created_at: row.try_get("created_at")?,
    })
}

pub async fn list(pool: &DbPool, project_id: &Uuid) -> Result<Vec<Experiment>, DomainError> {
    let rows = sqlx::query("SELECT id, project_id, name, description, scenario_type, tasks_json, global_config_json, created_at FROM experiments WHERE project_id = ? ORDER BY created_at DESC")
        .bind(project_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    rows.iter().map(row_to_experiment).collect()
}

pub async fn get(pool: &DbPool, id: &Uuid) -> Result<Experiment, DomainError> {
    let row = sqlx::query("SELECT id, project_id, name, description, scenario_type, tasks_json, global_config_json, created_at FROM experiments WHERE id = ?")
        .bind(id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    match row {
        Some(row) => row_to_experiment(&row),
        None => Err(DomainError::NotFound("experiment not found".into())),
    }
}

pub async fn create(pool: &DbPool, payload: NewExperiment) -> Result<Experiment, DomainError> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let tasks_str = serde_json::to_string(
        &payload
            .tasks
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>(),
    )
    .map_err(|e| DomainError::Internal(e.to_string()))?;
    let global_config_str = match payload.global_config {
        Some(ref value) => Some(
            serde_json::to_string(value).map_err(|e| DomainError::Internal(e.to_string()))?,
        ),
        None => None,
    };

    sqlx::query("INSERT INTO experiments (id, project_id, name, description, scenario_type, tasks_json, global_config_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(id.to_string())
        .bind(payload.project_id.to_string())
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(&payload.scenario_type)
        .bind(tasks_str)
        .bind(global_config_str)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    Ok(Experiment {
        id,
        project_id: payload.project_id,
        name: payload.name,
        description: payload.description,
        scenario_type: payload.scenario_type,
        tasks: payload.tasks,
        global_config: payload.global_config,
        created_at: now,
    })
}

