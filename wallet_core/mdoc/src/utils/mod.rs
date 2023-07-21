use chrono::{DateTime, Utc};

pub mod cose;
pub mod serialization;
pub mod signer;
pub mod x509;

pub(crate) mod crypto;

pub trait Generator<T> {
    fn generate(&self) -> T;
}

pub struct TimeGenerator;
impl Generator<DateTime<Utc>> for TimeGenerator {
    fn generate(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
