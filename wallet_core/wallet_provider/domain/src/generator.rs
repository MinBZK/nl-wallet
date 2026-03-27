use chrono::DateTime;
use chrono::Utc;
use utils::generator::Generator;
use uuid::Uuid;

pub struct Generators;

impl Generator<Uuid> for Generators {
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
}

impl Generator<DateTime<Utc>> for Generators {
    fn generate(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use chrono::DateTime;
    use chrono::Utc;
    use chrono::offset::TimeZone;
    use utils::generator::Generator;
    use uuid::Uuid;
    use uuid::uuid;

    pub struct FixedUuidGenerator;

    impl Generator<Uuid> for FixedUuidGenerator {
        fn generate(&self) -> Uuid {
            uuid!("c9723aef-022b-4ab7-9cc3-ff4227ec1cc9")
        }
    }

    pub struct EpochGenerator;

    impl Generator<DateTime<Utc>> for EpochGenerator {
        fn generate(&self) -> DateTime<Utc> {
            Utc.timestamp_nanos(0)
        }
    }
}
