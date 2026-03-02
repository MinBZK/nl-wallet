use http_utils::health::HealthChecker;

pub mod hsm;
pub mod postgres;

pub fn boxed(
    value: Option<impl HealthChecker + Send + Sync + 'static>,
) -> Option<Box<dyn HealthChecker + Send + Sync>> {
    value.map(|v| Box::new(v) as Box<_>)
}
