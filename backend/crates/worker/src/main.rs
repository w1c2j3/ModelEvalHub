use redis::AsyncCommands;
use tokio::time::{sleep, Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use unified_shared::eval::EvalConfig;
use unified_shared::settings::Settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let settings = Settings::load()?;
    let pool = deadpool_redis::Config::from_url(settings.redis.url.clone())
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

    loop {
        let mut conn = pool.get().await?;
        let job: Option<(String, String)> = conn
            .blpop(&settings.redis.queue_key, 5)
            .await
            .map_err(|err| anyhow::anyhow!(err))?;

        if let Some((_key, payload)) = job {
            tracing::info!("received job: {}", payload);
            let _config: EvalConfig = serde_json::from_str(&payload)?;
            // TODO: dispatch to integration adapters and result stores
        } else {
            sleep(Duration::from_secs(1)).await;
        }
    }
}

fn init_tracing() {
    let fmt_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry().with(fmt_layer).init();
}

