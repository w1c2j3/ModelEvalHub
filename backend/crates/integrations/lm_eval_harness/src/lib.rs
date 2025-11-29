use anyhow::Context;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command;
use unified_domain::result_store::ResultStore;
use unified_shared::eval::{EvalConfig, EvalResult};
use unified_shared::settings::Settings;

#[async_trait]
pub trait EvalRunner {
    async fn run(
        &self,
        config: &EvalConfig,
        result_store: &dyn ResultStore,
    ) -> anyhow::Result<()>;
}

pub struct LmEvalRunner {
    pub settings: Settings,
}

#[async_trait]
impl EvalRunner for LmEvalRunner {
    async fn run(
        &self,
        config: &EvalConfig,
        _result_store: &dyn ResultStore,
    ) -> anyhow::Result<()> {
        let run_dir = PathBuf::from(format!("runs/{}", config.run_id));
        tokio::fs::create_dir_all(&run_dir).await?;
        let config_path = run_dir.join("config.json");
        tokio::fs::write(&config_path, serde_json::to_vec_pretty(config)?).await?;

        let python = Command::new("python")
            .arg("-m")
            .arg("eval_runner")
            .arg("--run-dir")
            .arg(&run_dir)
            .env("EVAL_RUN_ID", config.run_id.to_string())
            .env("EVAL_RUN_DIR", &run_dir)
            .output()
            .await?;

        if python.status.success() {
            let result_path = run_dir.join("result.json");
            if result_path.exists() {
                let data = tokio::fs::read(result_path).await?;
                let _result: EvalResult = serde_json::from_slice(&data)?;
                // TODO: persist metrics & samples via result_store
            }
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&python.stderr);
            Err(anyhow::anyhow!("lm-eval runner failed: {stderr}"))
                .context("lm-eval-harness process failed")
        }
    }
}

