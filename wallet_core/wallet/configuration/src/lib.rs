pub mod config_server_config;
pub mod verification;
pub mod wallet_config;

pub trait EnvironmentSpecific {
    fn environment(&self) -> &str;
}
