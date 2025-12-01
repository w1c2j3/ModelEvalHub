use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

pub type DbPool = MySqlPool;

pub async fn init_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}
