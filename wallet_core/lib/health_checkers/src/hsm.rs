use async_trait::async_trait;

use hsm::service::Pkcs11Hsm;
use http_utils::health::HealthChecker;
use http_utils::health::HealthStatus;

#[derive(Debug, Clone)]
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

#[async_trait]
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
