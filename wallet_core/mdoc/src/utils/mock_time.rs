use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use parking_lot::RwLock;

use wallet_common::generator::Generator;

#[derive(Debug, Clone)]
pub struct MockTimeGenerator {
    pub time: Arc<RwLock<DateTime<Utc>>>,
}

impl MockTimeGenerator {
    pub fn new(time: DateTime<Utc>) -> Self {
        MockTimeGenerator {
            time: Arc::new(RwLock::new(time)),
        }
    }
}

impl Default for MockTimeGenerator {
    fn default() -> Self {
        MockTimeGenerator::new(Utc::now())
    }
}

impl Generator<DateTime<Utc>> for MockTimeGenerator {
    fn generate(&self) -> DateTime<Utc> {
        *self.time.read()
    }
}
