#[cfg(feature = "stub")]
pub mod stub {
    use chrono::{offset::TimeZone, DateTime, Local};
    use uuid::{uuid, Uuid};
    use wallet_common::generator::Generator;

    pub struct FixedGenerator;

    impl Generator<Uuid> for FixedGenerator {
        fn generate(&self) -> Uuid {
            uuid!("c9723aef-022b-4ab7-9cc3-ff4227ec1cc9")
        }
    }

    pub struct EpochGenerator;

    impl Generator<DateTime<Local>> for EpochGenerator {
        fn generate(&self) -> DateTime<Local> {
            Local.timestamp_nanos(0)
        }
    }
}
