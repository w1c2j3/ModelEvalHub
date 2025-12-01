use redis::AsyncCommands;
use tokio::time::{sleep, Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use unified_domain::db::DbPool;
use unified_domain::runs;
use unified_shared::eval::{EvalConfig, RunStatus};
use unified_shared::settings::Settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let settings = Settings::load()?;
    let redis_pool = deadpool_redis::Config::from_url(settings.redis.url.clone())
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    let db = unified_domain::db::init_pool(&settings.database.url).await?;

    loop {
        let mut conn = redis_pool.get().await?;
        let job: Option<(String, String)> = conn
            .blpop(&settings.redis.queue_key, 5)
            .await
            .map_err(|err| anyhow::anyhow!(err))?;

        if let Some((_key, payload)) = job {
            tracing::info!("received job payload");
            match serde_json::from_str::<EvalConfig>(&payload) {
                Ok(config) => {
                    if let Err(err) = process_job(&db, config).await {
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

async fn process_job(db: &DbPool, config: EvalConfig) -> anyhow::Result<()> {
    runs::update_status(db, &config.run_id, RunStatus::Running, None).await?;
    tracing::info!(
        "Executing run {:?} on engine {:?}",
        config.run_id,
        config.engine
    );
    sleep(Duration::from_millis(100)).await;
    runs::update_status(db, &config.run_id, RunStatus::Completed, None).await?;
    Ok(())
}

fn init_tracing() {
    let fmt_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry().with(fmt_layer).init();
}

