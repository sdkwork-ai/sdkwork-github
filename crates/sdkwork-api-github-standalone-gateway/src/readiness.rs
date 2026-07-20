use std::future::Future;
use std::pin::Pin;

use sdkwork_database_sqlx::DatabasePool;
use sdkwork_web_bootstrap::ReadinessCheck;

pub struct GithubDatabaseReadinessCheck {
    pool: DatabasePool,
}

impl GithubDatabaseReadinessCheck {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

impl ReadinessCheck for GithubDatabaseReadinessCheck {
    fn check(&self) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        let pool = self.pool.clone();
        Box::pin(async move { ping_database(&pool).await })
    }
}

async fn ping_database(pool: &DatabasePool) -> Result<(), String> {
    match pool {
        DatabasePool::Sqlite(sqlite, _) => sqlx::query("SELECT 1")
            .execute(sqlite)
            .await
            .map(|_| ())
            .map_err(|error| format!("sqlite ping failed: {error}")),
        DatabasePool::Postgres(postgres, _) => sqlx::query("SELECT 1")
            .execute(postgres)
            .await
            .map(|_| ())
            .map_err(|error| format!("postgres ping failed: {error}")),
    }
}
