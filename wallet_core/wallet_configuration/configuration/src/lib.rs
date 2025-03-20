pub mod config_server_config;
pub mod digid;
pub mod http;
pub mod wallet_config;

pub trait EnvironmentSpecific {
    fn environment(&self) -> &str;
}
