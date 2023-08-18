mod data;
mod local;

#[cfg(any(test, feature = "mock"))]
mod mock;

pub use self::{
    data::{AccountServerConfiguration, Configuration, LockTimeoutConfiguration, PidIssuanceConfiguration},
    local::LocalConfigurationRepository,
};

#[cfg(any(test, feature = "mock"))]
pub use self::mock::MockConfigurationRepository;

pub trait ConfigurationRepository {
    fn config(&self) -> &Configuration;
}
