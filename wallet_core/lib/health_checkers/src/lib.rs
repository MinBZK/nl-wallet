use http_utils::health::HealthChecker;

pub mod hsm;
pub mod postgres;

#[cfg(feature = "test_settings")]
pub mod test_settings;

pub fn boxed(value: Option<impl HealthChecker + 'static>) -> Option<Box<dyn HealthChecker>> {
    value.map(|v| Box::new(v) as Box<_>)
}
