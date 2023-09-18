mod data;
mod local;

pub use self::{
    data::{AccountServerConfiguration, Configuration, LockTimeoutConfiguration, PidIssuanceConfiguration},
    local::LocalConfigurationRepository,
};

pub trait ConfigurationRepository {
    fn config(&self) -> &Configuration;
}
