use std::env;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub queues: QueueSettings,
    pub integrations: IntegrationSettings,
    pub clickhouse: Option<ClickhouseSettings>,
    pub object_store: Option<ObjectStoreSettings>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisSettings {
    pub url: String,
    pub queue_key: String,
    pub dlq_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueueSettings {
    pub max_parallel_jobs: u32,
    pub max_parallel_gpu_jobs: u32,
    pub max_gpus_total: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationSettings {
    pub third_party_root: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClickhouseSettings {
    pub url: String,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub samples_table: String,
    pub metrics_table: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObjectStoreSettings {
    pub endpoint: String,
    pub region: Option<String>,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub use_path_style: bool,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(
                Environment::with_prefix("UEP")
                    .try_parsing(true)
                    .separator("__"),
            );

        if let Ok(env_name) = env::var("APP_ENV") {
            builder = builder
                .add_source(File::with_name(&format!("config/{}", env_name)).required(false));
        }

        builder.build()?.try_deserialize()
    }
}
