use async_trait::async_trait;
use unified_shared::eval::{MetricRecord, SampleRecord, SampleResultLocation};

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

