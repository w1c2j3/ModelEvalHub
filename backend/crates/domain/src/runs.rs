use crate::db::DbPool;
use crate::utils::parse_uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::mysql::MySqlRow;
use sqlx::Row;
use unified_shared::error::DomainError;
use unified_shared::eval::{EvalErrorKind, EvalErrorPayload, RunStatus};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: Uuid,
    pub experiment_id: Uuid,
    pub project_id: Uuid,
    pub model_impl_id: Uuid,
    pub checkpoint_id: Uuid,
    pub task_id: Uuid,
    pub run_type: String,
    pub status: RunStatus,
    pub error: Option<EvalErrorPayload>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub eval_config: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewRun {
    pub experiment_id: Uuid,
    pub project_id: Uuid,
    pub model_impl_id: Uuid,
    pub checkpoint_id: Uuid,
    pub task_id: Uuid,
    pub run_type: String,
    pub status: RunStatus,
    pub eval_config: Value,
}

fn status_to_str(status: RunStatus) -> &'static str {
    match status {
        RunStatus::Queued => "queued",
        RunStatus::Running => "running",
        RunStatus::Completed => "completed",
        RunStatus::FailedConfig => "failed_config",
        RunStatus::FailedEngine => "failed_engine",
        RunStatus::FailedInfra => "failed_infra",
        RunStatus::TimedOut => "timed_out",
        RunStatus::Cancelled => "cancelled",
    }
}

fn status_from_str(value: &str) -> RunStatus {
    match value {
        "queued" => RunStatus::Queued,
        "running" => RunStatus::Running,
        "completed" => RunStatus::Completed,
        "failed_config" => RunStatus::FailedConfig,
        "failed_engine" => RunStatus::FailedEngine,
        "failed_infra" => RunStatus::FailedInfra,
        "timed_out" => RunStatus::TimedOut,
        "cancelled" => RunStatus::Cancelled,
        _ => RunStatus::Queued,
    }
}

fn row_to_run(row: &MySqlRow) -> Result<Run, DomainError> {
    let eval_config: String = row.try_get("eval_config_json")?;
    let eval_value: Value =
        serde_json::from_str(&eval_config).map_err(|e| DomainError::Internal(e.to_string()))?;

    let error_kind = row.try_get::<Option<String>, _>("error_kind")?;
    let error_message = row.try_get::<Option<String>, _>("error_message")?;
    let error = error_kind.map(|kind| EvalErrorPayload {
        kind: match kind.as_str() {
            "config" => EvalErrorKind::Config,
            "engine" => EvalErrorKind::Engine,
            "infra" => EvalErrorKind::Infra,
            "timeout" => EvalErrorKind::Timeout,
            "cancelled" => EvalErrorKind::Cancelled,
            _ => EvalErrorKind::Unknown,
        },
        message: error_message.clone().unwrap_or_default(),
        code: row.try_get("error_code").ok().flatten(),
        engine: row.try_get("error_engine").ok().flatten(),
        details: row
            .try_get::<Option<String>, _>("error_details_json")
            .ok()
            .flatten()
            .map(|raw| serde_json::from_str(&raw).unwrap_or(Value::Null)),
    });

    Ok(Run {
        id: parse_uuid(row.try_get::<String, _>("id")?.as_str())?,
        experiment_id: parse_uuid(row.try_get::<String, _>("experiment_id")?.as_str())?,
        project_id: parse_uuid(row.try_get::<String, _>("project_id")?.as_str())?,
        model_impl_id: parse_uuid(row.try_get::<String, _>("model_impl_id")?.as_str())?,
        checkpoint_id: parse_uuid(row.try_get::<String, _>("checkpoint_id")?.as_str())?,
        task_id: parse_uuid(row.try_get::<String, _>("task_id")?.as_str())?,
        run_type: row.try_get("run_type")?,
        status: status_from_str(row.try_get::<String, _>("status")?.as_str()),
        error,
        started_at: row.try_get("started_at")?,
        finished_at: row.try_get("finished_at")?,
        eval_config: eval_value,
    })
}

pub async fn list(pool: &DbPool, project_id: &Uuid) -> Result<Vec<Run>, DomainError> {
    let rows = sqlx::query("SELECT id, experiment_id, project_id, model_impl_id, checkpoint_id, task_id, run_type, status, error_kind, error_code, error_message, error_engine, error_details_json, started_at, finished_at, eval_config_json FROM runs WHERE project_id = ? ORDER BY created_at DESC")
        .bind(project_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    rows.iter().map(row_to_run).collect()
}

pub async fn get(pool: &DbPool, id: &Uuid) -> Result<Run, DomainError> {
    let row = sqlx::query("SELECT id, experiment_id, project_id, model_impl_id, checkpoint_id, task_id, run_type, status, error_kind, error_code, error_message, error_engine, error_details_json, started_at, finished_at, eval_config_json FROM runs WHERE id = ?")
        .bind(id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    match row {
        Some(row) => row_to_run(&row),
        None => Err(DomainError::NotFound("run not found".into())),
    }
}

pub async fn create(pool: &DbPool, payload: NewRun) -> Result<Run, DomainError> {
    let id = Uuid::new_v4();
    let mut eval_config = payload.eval_config;
    if let Some(map) = eval_config.as_object_mut() {
        map.insert("run_id".into(), Value::String(id.to_string()));
        map.insert(
            "project_id".into(),
            Value::String(payload.project_id.to_string()),
        );
    }
    let eval_config_str =
        serde_json::to_string(&eval_config).map_err(|e| DomainError::Internal(e.to_string()))?;

    sqlx::query("INSERT INTO runs (id, experiment_id, project_id, model_impl_id, checkpoint_id, task_id, run_type, status, eval_config_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NOW())")
        .bind(id.to_string())
        .bind(payload.experiment_id.to_string())
        .bind(payload.project_id.to_string())
        .bind(payload.model_impl_id.to_string())
        .bind(payload.checkpoint_id.to_string())
        .bind(payload.task_id.to_string())
        .bind(&payload.run_type)
        .bind(status_to_str(payload.status))
        .bind(eval_config_str)
        .execute(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    Ok(Run {
        id,
        experiment_id: payload.experiment_id,
        project_id: payload.project_id,
        model_impl_id: payload.model_impl_id,
        checkpoint_id: payload.checkpoint_id,
        task_id: payload.task_id,
        run_type: payload.run_type,
        status: payload.status,
        error: None,
        started_at: None,
        finished_at: None,
        eval_config,
    })
}

pub async fn update_status(
    pool: &DbPool,
    id: &Uuid,
    status: RunStatus,
    error: Option<EvalErrorPayload>,
) -> Result<(), DomainError> {
    sqlx::query(
        "UPDATE runs SET status = ?, error_kind = ?, error_code = ?, error_message = ?, error_engine = ?, error_details_json = ?, updated_at = NOW() WHERE id = ?",
    )
    .bind(status_to_str(status))
    .bind(error.as_ref().map(|e| format!("{:?}", e.kind).to_lowercase()))
    .bind(error.as_ref().and_then(|e| e.code.clone()))
    .bind(error.as_ref().map(|e| e.message.clone()))
    .bind(error.as_ref().and_then(|e| e.engine.clone()))
    .bind(
        error
            .as_ref()
            .and_then(|e| e.details.clone())
            .map(|d| serde_json::to_string(&d).unwrap_or_else(|_| "{}".into())),
    )
    .bind(id.to_string())
    .execute(pool)
    .await
    .map_err(|e| DomainError::Internal(e.to_string()))?;

    Ok(())
}

