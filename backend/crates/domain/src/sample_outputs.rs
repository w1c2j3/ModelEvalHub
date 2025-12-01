use crate::db::DbPool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use unified_shared::error::DomainError;
use unified_shared::eval::{SampleRecord, SampleResultLocation};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleOutput {
    pub id: Uuid,
    pub run_id: Uuid,
    pub dataset: String,
    pub subset: Option<String>,
    pub split: Option<String>,
    pub sample_index: i64,
    pub input: String,
    pub reference: Option<String>,
    pub output: String,
    pub metrics: Option<Value>,
    pub latency_ms: Option<i64>,
    pub token_counts: Option<Value>,
    pub error: Option<Value>,
    pub created_at: DateTime<Utc>,
}

pub async fn list_by_run(pool: &DbPool, run_id: &Uuid) -> Result<Vec<SampleOutput>, DomainError> {
    let rows = sqlx::query("SELECT id, run_id, dataset, subset, split, sample_index, input_text, reference_text, output_text, metrics_json, latency_ms, token_counts_json, error_json, created_at FROM sample_outputs WHERE run_id = ? ORDER BY sample_index ASC")
        .bind(run_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    let mut samples = Vec::new();
    for row in rows {
        let metrics = row
            .try_get::<Option<String>, _>("metrics_json")
            .ok()
            .flatten()
            .map(|raw| serde_json::from_str(&raw).unwrap_or(Value::Null));
        let token_counts = row
            .try_get::<Option<String>, _>("token_counts_json")
            .ok()
            .flatten()
            .map(|raw| serde_json::from_str(&raw).unwrap_or(Value::Null));
        let error = row
            .try_get::<Option<String>, _>("error_json")
            .ok()
            .flatten()
            .map(|raw| serde_json::from_str(&raw).unwrap_or(Value::Null));

        samples.push(SampleOutput {
            id: Uuid::parse_str(row.try_get::<String, _>("id")?.as_str())
                .map_err(|e| DomainError::Internal(e.to_string()))?,
            run_id: Uuid::parse_str(row.try_get::<String, _>("run_id")?.as_str())
                .map_err(|e| DomainError::Internal(e.to_string()))?,
            dataset: row.try_get("dataset")?,
            subset: row.try_get("subset")?,
            split: row.try_get("split")?,
            sample_index: row.try_get("sample_index")?,
            input: row.try_get("input_text")?,
            reference: row.try_get("reference_text")?,
            output: row.try_get("output_text")?,
            metrics,
            latency_ms: row.try_get("latency_ms")?,
            token_counts,
            error,
            created_at: row.try_get("created_at")?,
        });
    }

    Ok(samples)
}

pub async fn save_inline(
    pool: &DbPool,
    records: &[SampleRecord],
) -> Result<SampleResultLocation, DomainError> {
    for record in records {
        sqlx::query("INSERT INTO sample_outputs (id, run_id, dataset, subset, split, sample_index, input_text, reference_text, output_text, metrics_json, latency_ms, token_counts_json, error_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind(record.run_id.to_string())
            .bind(&record.dataset)
            .bind(&record.subset)
            .bind(&record.split)
            .bind(record.sample_index)
            .bind(&record.input)
            .bind(&record.reference)
            .bind(&record.output)
            .bind(record.metrics.as_ref().map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".into())))
            .bind(record.latency_ms)
            .bind(record.token_counts.as_ref().map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".into())))
            .bind(record.error.as_ref().map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".into())))
            .bind(Utc::now())
            .execute(pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
    }

    Ok(SampleResultLocation::Inline {
        samples: records.to_vec(),
    })
}
