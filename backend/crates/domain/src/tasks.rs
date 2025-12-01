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
pub struct Task {
    pub id: Uuid,
    pub project_id: Uuid,
    pub dataset_id: Uuid,
    pub name: String,
    pub task_type: String,
    pub eval_engine: String,
    pub eval_config: Value,
    pub default_metrics: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewTask {
    pub project_id: Uuid,
    pub dataset_id: Uuid,
    pub name: String,
    pub task_type: String,
    pub eval_engine: String,
    pub eval_config: Value,
    pub default_metrics: Option<Value>,
}

fn row_to_task(row: &MySqlRow) -> Result<Task, DomainError> {
    let eval_config: String = row.try_get("eval_config_json")?;
    let eval_value: Value =
        serde_json::from_str(&eval_config).map_err(|e| DomainError::Internal(e.to_string()))?;
    let metrics_value = row
        .try_get::<Option<String>, _>("default_metrics_json")?
        .map(|raw| serde_json::from_str(&raw).unwrap_or(Value::Null));

    Ok(Task {
        id: parse_uuid(row.try_get::<String, _>("id")?.as_str())?,
        project_id: parse_uuid(row.try_get::<String, _>("project_id")?.as_str())?,
        dataset_id: parse_uuid(row.try_get::<String, _>("dataset_id")?.as_str())?,
        name: row.try_get("name")?,
        task_type: row.try_get("task_type")?,
        eval_engine: row.try_get("eval_engine")?,
        eval_config: eval_value,
        default_metrics: metrics_value,
        created_at: row.try_get("created_at")?,
    })
}

pub async fn list(pool: &DbPool, project_id: &Uuid) -> Result<Vec<Task>, DomainError> {
    let rows = sqlx::query("SELECT id, project_id, dataset_id, name, task_type, eval_engine, eval_config_json, default_metrics_json, created_at FROM tasks WHERE project_id = ? ORDER BY created_at DESC")
        .bind(project_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    rows.iter().map(row_to_task).collect()
}

pub async fn create(pool: &DbPool, payload: NewTask) -> Result<Task, DomainError> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let eval_config_str = serde_json::to_string(&payload.eval_config)
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let default_metrics_str = match payload.default_metrics {
        Some(ref value) => {
            Some(serde_json::to_string(value).map_err(|e| DomainError::Internal(e.to_string()))?)
        }
        None => None,
    };

    sqlx::query("INSERT INTO tasks (id, project_id, dataset_id, name, task_type, eval_engine, eval_config_json, default_metrics_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(id.to_string())
        .bind(payload.project_id.to_string())
        .bind(payload.dataset_id.to_string())
        .bind(&payload.name)
        .bind(&payload.task_type)
        .bind(&payload.eval_engine)
        .bind(eval_config_str)
        .bind(default_metrics_str)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    Ok(Task {
        id,
        project_id: payload.project_id,
        dataset_id: payload.dataset_id,
        name: payload.name,
        task_type: payload.task_type,
        eval_engine: payload.eval_engine,
        eval_config: payload.eval_config,
        default_metrics: payload.default_metrics,
        created_at: now,
    })
}
