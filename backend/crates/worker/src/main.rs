use integration_lm_eval_harness::{LmEvalRunner, RunnerError};
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use unified_domain::db::DbPool;
use unified_domain::result_store::ResultStoreHandles;
use unified_domain::runs;
use unified_shared::eval::{EvalConfig, EvalEngine, EvalErrorKind, EvalErrorPayload, RunStatus};
use unified_shared::settings::Settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let settings = Settings::load()?;
    let redis_pool = deadpool_redis::Config::from_url(settings.redis.url.clone())
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    let db = unified_domain::db::init_pool(&settings.database.url).await?;
    let stores = ResultStoreHandles::new(&settings, db.clone()).await?;
    let runners = Runners::new(&settings);
    let ctx = Arc::new(WorkerContext {
        settings,
        db,
        stores,
        runners,
    });

    loop {
        let mut conn = redis_pool.get().await?;
        let job: Option<(String, String)> = conn
            .blpop(&ctx.settings.redis.queue_key, 5)
            .await
            .map_err(|err| anyhow::anyhow!(err))?;

        if let Some((_key, payload)) = job {
            tracing::info!("received job payload");
            match serde_json::from_str::<EvalConfig>(&payload) {
                Ok(config) => {
                    if let Err(err) = process_job(ctx.clone(), config).await {
                        tracing::error!("job failed: {err:?}");
                    }
                }
                Err(err) => tracing::error!("invalid job payload: {err:?}"),
            }
        } else {
            sleep(Duration::from_secs(1)).await;
        }
    }
}

struct WorkerContext {
    settings: Settings,
    db: DbPool,
    stores: ResultStoreHandles,
    runners: Runners,
}

struct Runners {
    lm_eval: integration_lm_eval_harness::LmEvalRunner,
}

impl Runners {
    fn new(settings: &Settings) -> Self {
        Self {
            lm_eval: integration_lm_eval_harness::LmEvalRunner::new(settings),
        }
    }
}

async fn process_job(ctx: Arc<WorkerContext>, config: EvalConfig) -> anyhow::Result<()> {
    runs::update_status(&ctx.db, &config.run_id, RunStatus::Running, None).await?;
    tracing::info!("running job {} via {:?}", config.run_id, config.engine);

    let result = match config.engine {
        EvalEngine::LmEvalHarness => ctx.runners.lm_eval.run(&config).await,
        other => {
            tracing::warn!("engine {:?} not supported yet", other);
            Err(integration_lm_eval_harness::RunnerError::NotSupported)
        }
    };

    match result {
        Ok(eval_result) => {
            ctx.stores
                .persist_eval_result(&config, &eval_result)
                .await?;
            runs::update_status(&ctx.db, &config.run_id, RunStatus::Completed, None).await?;
        }
        Err(err) => {
            let payload = match err {
                integration_lm_eval_harness::RunnerError::Eval(payload) => payload,
                integration_lm_eval_harness::RunnerError::Io(io_err) => EvalErrorPayload {
                    kind: EvalErrorKind::Infra,
                    message: io_err.to_string(),
                    code: None,
                    engine: Some("lm_eval_harness".into()),
                    details: None,
                },
                integration_lm_eval_harness::RunnerError::NotSupported => EvalErrorPayload {
                    kind: EvalErrorKind::Engine,
                    message: "Engine not supported".into(),
                    code: None,
                    engine: Some(format!("{:?}", config.engine)),
                    details: None,
                },
            };
            let status = map_error_to_status(payload.kind);
            runs::update_status(&ctx.db, &config.run_id, status, Some(payload)).await?;
        }
    }

    Ok(())
}

fn map_error_to_status(kind: EvalErrorKind) -> RunStatus {
    match kind {
        EvalErrorKind::Config => RunStatus::FailedConfig,
        EvalErrorKind::Engine => RunStatus::FailedEngine,
        EvalErrorKind::Infra => RunStatus::FailedInfra,
        EvalErrorKind::Timeout => RunStatus::TimedOut,
        EvalErrorKind::Cancelled => RunStatus::Cancelled,
        EvalErrorKind::Unknown => RunStatus::FailedInfra,
    }
}

fn init_tracing() {
    let fmt_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry().with(fmt_layer).init();
}
