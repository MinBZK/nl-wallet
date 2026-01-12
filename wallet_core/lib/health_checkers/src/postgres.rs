use sea_orm::DatabaseConnection;
use sea_orm::sqlx::Connection;
use sea_orm::sqlx::PgPool;

use http_utils::health::HealthChecker;
use http_utils::health::HealthStatus;

#[derive(Debug, Clone)]
pub struct DatabaseChecker {
    name: &'static str,
    pool: PgPool,
    check: bool,
}

impl DatabaseChecker {
    pub fn new(name: &'static str, connection: &DatabaseConnection) -> Self {
        let pool = connection.get_postgres_connection_pool().clone();
        let check = !pool.options().get_test_before_acquire();

        Self { name, pool, check }
    }

    pub fn rename(&mut self, name: &'static str) {
        self.name = name;
    }
}

#[async_trait::async_trait]
impl HealthChecker for DatabaseChecker {
    fn name(&self) -> &'static str {
        self.name
    }

    async fn status(&self) -> Result<HealthStatus, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut conn = self.pool.acquire().await?;
        if self.check {
            conn.ping().await?;
        }

        Ok(HealthStatus::UP)
    }
}
