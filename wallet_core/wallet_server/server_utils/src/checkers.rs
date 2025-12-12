#[cfg(feature = "postgres")]
mod postgres {
    use sea_orm::DatabaseConnection;
    use sea_orm::sqlx::Connection;
    use sea_orm::sqlx::PgPool;

    use http_utils::health::HealthChecker;
    use http_utils::health::HealthStatus;

    #[derive(Clone)]
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
}

#[cfg(feature = "postgres")]
pub use postgres::DatabaseChecker;

use hsm::service::Pkcs11Hsm;
use http_utils::health::HealthChecker;
use http_utils::health::HealthStatus;

#[derive(Clone)]
pub struct HsmChecker {
    pool: r2d2_cryptoki::Pool,
    check: bool,
}
impl HsmChecker {
    pub fn new(hsm: &Pkcs11Hsm) -> Self {
        let pool = hsm.as_ref().clone();
        let check = !pool.test_on_check_out();
        Self { pool, check }
    }
}

impl From<HsmChecker> for Box<dyn HealthChecker> {
    fn from(value: HsmChecker) -> Self {
        Box::new(value)
    }
}

#[async_trait::async_trait]
impl HealthChecker for HsmChecker {
    fn name(&self) -> &'static str {
        "hsm"
    }

    async fn status(&self) -> Result<HealthStatus, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let conn = self.pool.get()?;
        if self.check {
            conn.get_session_info()?;
        }
        Ok(HealthStatus::UP)
    }
}

pub fn boxed(value: Option<impl HealthChecker + 'static>) -> Option<Box<dyn HealthChecker>> {
    value.map(|v| Box::new(v) as Box<_>)
}
