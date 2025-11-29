use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use unified_shared::settings::Settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let settings = Settings::load()?;

    let app = Router::new().route("/healthz", get(health_check));

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    tracing::debug!("settings loaded: {:?}", settings.integrations.third_party_root);
    Ok(())
}

async fn health_check() -> &'static str {
    "ok"
}

fn init_tracing() {
    let fmt_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry().with(fmt_layer).init();
}

