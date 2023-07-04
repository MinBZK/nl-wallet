pub trait Generator<T> {
    fn generate(&self) -> T;
}

#[cfg(feature = "stub")]
pub mod stub {
    use uuid::{uuid, Uuid};

    use super::Generator;

    pub struct FixedGenerator;

    impl Generator<Uuid> for FixedGenerator {
        fn generate(&self) -> Uuid {
            uuid!("c9723aef-022b-4ab7-9cc3-ff4227ec1cc9")
        }
    }
}
