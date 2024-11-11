use chrono::{DateTime, Utc};

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
