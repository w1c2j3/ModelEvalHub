use crate::db::DbPool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use unified_shared::error::DomainError;
use unified_shared::eval::MetricRecord;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub id: Uuid,
    pub run_id: Uuid,
    pub dataset: String,
    pub subset: Option<String>,
    pub split: Option<String>,
    pub metric_name: String,
    pub value: f64,
    pub n_samples: Option<i64>,
    pub ci_low: Option<f64>,
    pub ci_high: Option<f64>,
    pub extra: Option<Value>,
    pub timestamp: DateTime<Utc>,
}

pub async fn list_by_run(pool: &DbPool, run_id: &Uuid) -> Result<Vec<Metric>, DomainError> {
    let rows = sqlx::query("SELECT id, run_id, dataset, subset, split, metric_name, value, n_samples, ci_low, ci_high, extra_json, timestamp FROM metrics WHERE run_id = ? ORDER BY timestamp ASC")
        .bind(run_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    let mut metrics = Vec::new();
    for row in rows {
        let extra = row
            .try_get::<Option<String>, _>("extra_json")
            .ok()
            .flatten()
            .map(|raw| serde_json::from_str(&raw).unwrap_or(Value::Null));
        metrics.push(Metric {
            id: Uuid::parse_str(row.try_get::<String, _>("id")?.as_str())
                .map_err(|e| DomainError::Internal(e.to_string()))?,
            run_id: Uuid::parse_str(row.try_get::<String, _>("run_id")?.as_str())
                .map_err(|e| DomainError::Internal(e.to_string()))?,
            dataset: row.try_get("dataset")?,
            subset: row.try_get("subset")?,
            split: row.try_get("split")?,
            metric_name: row.try_get("metric_name")?,
            value: row.try_get("value")?,
            n_samples: row.try_get("n_samples")?,
            ci_low: row.try_get("ci_low")?,
            ci_high: row.try_get("ci_high")?,
            extra,
            timestamp: row.try_get("timestamp")?,
        });
    }

    Ok(metrics)
}

pub async fn save_records(pool: &DbPool, records: &[MetricRecord]) -> Result<(), DomainError> {
    for record in records {
        sqlx::query("INSERT INTO metrics (id, run_id, dataset, subset, split, metric_name, value, n_samples, ci_low, ci_high, extra_json, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind(record.run_id.to_string())
            .bind(&record.dataset)
            .bind(&record.subset)
            .bind(&record.split)
            .bind(&record.metric_name)
            .bind(record.value)
            .bind(record.n_samples)
            .bind(record.ci_low)
            .bind(record.ci_high)
            .bind(record.extra.as_ref().map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".into())))
            .bind(Utc::now())
            .execute(pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
    }
    Ok(())
}
