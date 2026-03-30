use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

pub trait Generator<T> {
    fn generate(&self) -> T;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TimeGenerator;

impl Generator<DateTime<Utc>> for TimeGenerator {
    fn generate(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UuidV4AndTimeGenerator;

impl Generator<DateTime<Utc>> for UuidV4AndTimeGenerator {
    fn generate(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

impl Generator<Uuid> for UuidV4AndTimeGenerator {
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
}

#[cfg(any(test, feature = "mock_time"))]
pub mod mock {
    use std::sync::Arc;

    use chrono::DateTime;
    use chrono::Utc;
    use chrono::offset::TimeZone;
    use parking_lot::RwLock;

    use super::Generator;

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

        pub fn epoch() -> Self {
            MockTimeGenerator::new(Utc.timestamp_nanos(0))
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
}
