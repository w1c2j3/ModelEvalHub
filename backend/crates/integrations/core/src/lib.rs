use anyhow::Result;
use async_trait::async_trait;
use unified_shared::eval::EvalConfig;
use unified_shared::eval::EvalResult;

#[async_trait]
pub trait EvalRunner: Send + Sync {
    async fn run(&self, config: &EvalConfig) -> Result<EvalResult>;
    fn name(&self) -> &'static str;
}

