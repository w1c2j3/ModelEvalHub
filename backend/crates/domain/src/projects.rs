use crate::db::DbPool;
use crate::utils::parse_uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlRow;
use sqlx::Row;
use unified_shared::error::DomainError;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewProject {
    pub name: String,
    pub description: Option<String>,
}

fn row_to_project(row: &MySqlRow) -> Result<Project, DomainError> {
    Ok(Project {
        id: parse_uuid(row.try_get::<String, _>("id")?.as_str())?,
        name: row.try_get("name")?,
        description: row.try_get::<Option<String>, _>("description")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub async fn list(pool: &DbPool) -> Result<Vec<Project>, DomainError> {
    let rows = sqlx::query("SELECT id, name, description, created_at, updated_at FROM projects ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    rows.iter().map(row_to_project).collect()
}

pub async fn get(pool: &DbPool, id: &Uuid) -> Result<Project, DomainError> {
    let row = sqlx::query(
        "SELECT id, name, description, created_at, updated_at FROM projects WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| DomainError::Internal(e.to_string()))?;

    match row {
        Some(row) => row_to_project(&row),
        None => Err(DomainError::NotFound("project not found".into())),
    }
}

pub async fn create(pool: &DbPool, payload: NewProject) -> Result<Project, DomainError> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| DomainError::Internal(e.to_string()))?;

    Ok(Project {
        id,
        name: payload.name,
        description: payload.description,
        created_at: now,
        updated_at: now,
    })
}
