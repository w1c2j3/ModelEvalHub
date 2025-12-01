use anyhow::Context;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::process::Command;
use unified_shared::eval::{EvalConfig, EvalErrorPayload, EvalResult};
use unified_shared::settings::Settings;

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("evaluation failure")]
    Eval(EvalErrorPayload),
    #[error(transparent)]
    Io(#[from] anyhow::Error),
    #[error("engine not supported")]
    NotSupported,
}

pub struct LmEvalRunner {
    harness_root: PathBuf,
}

impl LmEvalRunner {
    pub fn new(settings: &Settings) -> Self {
        let root = Path::new(&settings.integrations.third_party_root).join("lm-evaluation-harness");
        Self { harness_root: root }
    }

    pub async fn run(&self, config: &EvalConfig) -> Result<EvalResult, RunnerError> {
        let run_dir = PathBuf::from(format!("runs/{}", config.run_id));
        tokio::fs::create_dir_all(&run_dir).await?;
        let config_path = run_dir.join("config.json");
        tokio::fs::write(&config_path, serde_json::to_vec_pretty(config)?).await?;

        let mut cmd = Command::new("python");
        cmd.arg("-m")
            .arg("eval_runner")
            .arg("--run-dir")
            .arg(&run_dir)
            .env("EVAL_RUN_ID", config.run_id.to_string())
            .env("EVAL_RUN_DIR", &run_dir);
        if self.harness_root.exists() {
            cmd.current_dir(&self.harness_root);
        }

        let output = cmd.output().await?;
        if output.status.success() {
            let result_path = run_dir.join("result.json");
            if result_path.exists() {
                let data = tokio::fs::read(result_path).await?;
                let result: EvalResult =
                    serde_json::from_slice(&data).context("invalid eval result json")?;
                Ok(result)
            } else {
                Err(anyhow::anyhow!("result.json missing").into())
            }
        } else {
            let error_path = run_dir.join("error.json");
            if error_path.exists() {
                let data = tokio::fs::read(error_path).await?;
                let payload: EvalErrorPayload =
                    serde_json::from_slice(&data).context("invalid error payload")?;
                Err(RunnerError::Eval(payload))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(RunnerError::Io(anyhow::anyhow!(
                    "lm-eval harness failed: {stderr}"
                )))
            }
        }
    }
}
