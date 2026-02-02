pub mod hsm;
pub mod pin_policy;
pub mod wallet_user;

#[cfg(feature = "mock")]
pub use self::pin_policy::mock::FailingPinPolicy;
#[cfg(feature = "mock")]
pub use self::pin_policy::mock::TimeoutPinPolicy;

#[derive(Debug, derive_more::Unwrap)]
pub enum QueryResult<T> {
    Found(Box<T>),
    NotFound,
}
