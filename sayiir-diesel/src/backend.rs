use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;

#[cfg(feature = "sqlite")]
type Connection = diesel_async::sync_connection_wrapper::SyncConnectionWrapper<diesel::SqliteConnection>;

#[cfg(feature = "postgres")]
type Connection = diesel_async::AsyncPgConnection;

#[cfg(feature = "mysql")]
type Connection = diesel_async::AsyncMysqlConnection;

use crate::{DieselError, Result};

/// Diesel-backed persistence for sayiir workflows
pub struct DieselBackend {
    pool: Pool<Connection>,
}

impl DieselBackend {
    /// Create a new DieselBackend with connection pool and run migrations
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = create_pool(database_url).await?;
        run_migrations(&pool).await?;
        Ok(Self { pool })
    }
}

async fn create_pool(database_url: &str) -> Result<Pool<Connection>> {
    let config = AsyncDieselConnectionManager::<Connection>::new(database_url);
    let pool = Pool::builder(config)
        .build()
        .map_err(|e| DieselError::ConnectionError(e.to_string()))?;
    Ok(pool)
}

async fn run_migrations(_pool: &Pool<Connection>) -> Result<()> {
    // TODO: Implement migration runner in Task 1.4
    Ok(())
}
