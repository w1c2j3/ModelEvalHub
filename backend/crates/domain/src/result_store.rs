use std::io::Write;
use std::sync::Arc;

use anyhow::bail;
use anyhow::bail;
use async_trait::async_trait;
use clickhouse::{Client as ClickHouseClient, Row};
use s3::creds::Credentials;
use s3::{Bucket, Region};
use unified_shared::eval::{
    EvalConfig, EvalResult, MetricRecord, OutputConfig, SampleRecord, SampleResultLocation,
};
use unified_shared::settings::{ClickhouseSettings, ObjectStoreSettings};
use uuid::Uuid;

#[async_trait]
pub trait ResultStore: Send + Sync {
    async fn save_metrics(&self, records: &[MetricRecord]) -> anyhow::Result<()>;
    async fn save_samples_inline(
        &self,
        records: &[SampleRecord],
    ) -> anyhow::Result<SampleResultLocation>;
    async fn save_samples_location(
        &self,
        run_id: uuid::Uuid,
        location: &SampleResultLocation,
    ) -> anyhow::Result<()>;
}

pub struct DbResultStore {
    pub db: crate::db::DbPool,
}

#[async_trait]
impl ResultStore for DbResultStore {
    async fn save_metrics(&self, records: &[MetricRecord]) -> anyhow::Result<()> {
        crate::metrics::save_records(&self.db, records).await?;
        Ok(())
    }

    async fn save_samples_inline(
        &self,
        records: &[SampleRecord],
    ) -> anyhow::Result<SampleResultLocation> {
        crate::sample_outputs::save_inline(&self.db, records).await
    }

    async fn save_samples_location(
        &self,
        _run_id: Uuid,
        _location: &SampleResultLocation,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct ClickHouseResultStore {
    pub client: ClickHouseClient,
    pub settings: ClickhouseSettings,
}

#[async_trait]
impl ResultStore for ClickHouseResultStore {
    async fn save_metrics(&self, records: &[MetricRecord]) -> anyhow::Result<()> {
        #[derive(Row)]
        struct MetricRow<'a> {
            run_id: &'a str,
            dataset: &'a str,
            subset: Option<&'a str>,
            split: Option<&'a str>,
            metric_name: &'a str,
            value: f64,
            n_samples: Option<i64>,
            ci_low: Option<f64>,
            ci_high: Option<f64>,
            extra_json: Option<&'a str>,
        }

        let mut insert = self.client.insert(&self.settings.metrics_table).await?;
        for record in records {
            insert
                .write(&MetricRow {
                    run_id: &record.run_id.to_string(),
                    dataset: &record.dataset,
                    subset: record.subset.as_deref(),
                    split: record.split.as_deref(),
                    metric_name: &record.metric_name,
                    value: record.value,
                    n_samples: record.n_samples,
                    ci_low: record.ci_low,
                    ci_high: record.ci_high,
                    extra_json: record
                        .extra
                        .as_ref()
                        .map(|v| serde_json::to_string(v).unwrap_or_default())
                        .as_deref(),
                })
                .await?;
        }
        insert.end().await?;
        Ok(())
    }

    async fn save_samples_inline(
        &self,
        records: &[SampleRecord],
    ) -> anyhow::Result<SampleResultLocation> {
        #[derive(Row)]
        struct SampleRow<'a> {
            run_id: &'a str,
            dataset: &'a str,
            subset: Option<&'a str>,
            split: Option<&'a str>,
            sample_index: i64,
            input: &'a str,
            reference: Option<&'a str>,
            output: &'a str,
            metrics_json: Option<&'a str>,
            latency_ms: Option<i64>,
            token_counts_json: Option<&'a str>,
            error_json: Option<&'a str>,
        }

        let mut insert = self.client.insert(&self.settings.samples_table).await?;
        for record in records {
            insert
                .write(&SampleRow {
                    run_id: &record.run_id.to_string(),
                    dataset: &record.dataset,
                    subset: record.subset.as_deref(),
                    split: record.split.as_deref(),
                    sample_index: record.sample_index,
                    input: &record.input,
                    reference: record.reference.as_deref(),
                    output: &record.output,
                    metrics_json: record
                        .metrics
                        .as_ref()
                        .map(|v| serde_json::to_string(v).unwrap_or_default())
                        .as_deref(),
                    latency_ms: record.latency_ms,
                    token_counts_json: record
                        .token_counts
                        .as_ref()
                        .map(|v| serde_json::to_string(v).unwrap_or_default())
                        .as_deref(),
                    error_json: record
                        .error
                        .as_ref()
                        .map(|v| serde_json::to_string(v).unwrap_or_default())
                        .as_deref(),
                })
                .await?;
        }
        insert.end().await?;

        Ok(SampleResultLocation::ClickHouse {
            table: self.settings.samples_table.clone(),
        })
    }

    async fn save_samples_location(
        &self,
        _run_id: Uuid,
        _location: &SampleResultLocation,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct ObjectStoreResultStore {
    pub settings: ObjectStoreSettings,
    pub bucket: Bucket,
}

impl ObjectStoreResultStore {
    pub fn new(settings: ObjectStoreSettings) -> anyhow::Result<Self> {
        let region = settings
            .region
            .clone()
            .map(Region::from)
            .unwrap_or_else(|| Region::new("us-east-1"));
        let credentials = Credentials::new(
            Some(&settings.access_key),
            Some(&settings.secret_key),
            None,
            None,
            None,
        )?;
        let mut bucket = Bucket::new(&settings.bucket, region, credentials)?;
        bucket = if settings.use_path_style {
            bucket.with_path_style()
        } else {
            bucket
        };
        bucket.set_endpoint(&settings.endpoint)?;
        Ok(Self { settings, bucket })
    }
}

#[async_trait]
impl ResultStore for ObjectStoreResultStore {
    async fn save_metrics(&self, records: &[MetricRecord]) -> anyhow::Result<()> {
        // still rely on DB for metrics if object store only used for samples
        anyhow::bail!("ObjectStoreResultStore does not support metrics");
    }

    async fn save_samples_inline(
        &self,
        records: &[SampleRecord],
    ) -> anyhow::Result<SampleResultLocation> {
        let run_id = records
            .first()
            .map(|r| r.run_id)
            .unwrap_or_else(Uuid::new_v4);
        let key = format!("runs/{run_id}/samples.jsonl");
        let mut body = Vec::new();
        for record in records {
            writeln!(
                body,
                "{}",
                serde_json::to_string(record).unwrap_or_else(|_| "{}".into())
            )?;
        }
        let (_, code) = self.bucket.put_object(&key, &body).await?;
        if code >= 300 {
            bail!("failed to upload samples to object store (status {code})");
        }
        let base = format!(
            "{}/{}",
            self.settings.endpoint.trim_end_matches('/'),
            self.settings.bucket
        );

        Ok(SampleResultLocation::ObjectStore {
            uri: format!("{}/{}", base, key),
            format: "jsonl".into(),
        })
    }

    async fn save_samples_location(
        &self,
        _run_id: Uuid,
        _location: &SampleResultLocation,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct ResultStoreHandles {
    pub db: Arc<DbResultStore>,
    pub clickhouse: Option<Arc<ClickHouseResultStore>>,
    pub object_store: Option<Arc<ObjectStoreResultStore>>,
}

impl ResultStoreHandles {
    pub async fn new(
        settings: &unified_shared::settings::Settings,
        db: crate::db::DbPool,
    ) -> anyhow::Result<Self> {
        let db_store = Arc::new(DbResultStore { db });
        let clickhouse = settings.clickhouse.as_ref().map(|cfg| {
            Arc::new(ClickHouseResultStore {
                client: ClickHouseClient::default()
                    .with_url(&cfg.url)
                    .with_database(&cfg.database)
                    .with_user(
                        cfg.username.clone().unwrap_or_else(|| "default".into()),
                        cfg.password.clone().unwrap_or_default(),
                    ),
                settings: cfg.clone(),
            })
        });

        let object_store = match settings.object_store.clone() {
            Some(cfg) => Some(Arc::new(ObjectStoreResultStore::new(cfg.clone())?)),
            None => None,
        };

        Ok(ResultStoreHandles {
            db: db_store,
            clickhouse,
            object_store,
        })
    }

    pub async fn persist_eval_result(
        &self,
        config: &EvalConfig,
        result: &EvalResult,
    ) -> anyhow::Result<()> {
        self.save_metrics(config, result).await?;
        self.save_samples(config, result).await?;
        Ok(())
    }

    async fn save_metrics(&self, config: &EvalConfig, result: &EvalResult) -> anyhow::Result<()> {
        match config.output {
            OutputConfig::ClickHouse { .. } => {
                if let Some(ch) = &self.clickhouse {
                    ch.save_metrics(&result.metrics).await
                } else {
                    self.db.save_metrics(&result.metrics).await
                }
            }
            OutputConfig::Hybrid { .. }
            | OutputConfig::DbOnly
            | OutputConfig::ObjectStore { .. } => self.db.save_metrics(&result.metrics).await,
        }
    }

    async fn save_samples(&self, config: &EvalConfig, result: &EvalResult) -> anyhow::Result<()> {
        match (&config.output, &result.samples) {
            (_, SampleResultLocation::Inline { samples }) => match config.output {
                OutputConfig::DbOnly => {
                    self.db.save_samples_inline(samples).await?;
                }
                OutputConfig::ObjectStore { .. } => {
                    if let Some(obj) = &self.object_store {
                        obj.save_samples_inline(samples).await?;
                    } else {
                        self.db.save_samples_inline(samples).await?;
                    }
                }
                OutputConfig::ClickHouse { .. } => {
                    if let Some(ch) = &self.clickhouse {
                        ch.save_samples_inline(samples).await?;
                    } else {
                        self.db.save_samples_inline(samples).await?;
                    }
                }
                OutputConfig::Hybrid { .. } => {
                    if let Some(ch) = &self.clickhouse {
                        ch.save_samples_inline(samples).await?;
                    } else {
                        self.db.save_samples_inline(samples).await?;
                    }
                }
            },
            (_, location) => {
                self.db
                    .save_samples_location(result.run_id, location)
                    .await?;
            }
        }

        Ok(())
    }
}
