use http_utils::health::HealthChecker;

pub mod hsm;
pub mod postgres;

pub fn boxed(value: Option<impl HealthChecker + 'static>) -> Option<Box<dyn HealthChecker>> {
    value.map(|v| Box::new(v) as Box<_>)
}
