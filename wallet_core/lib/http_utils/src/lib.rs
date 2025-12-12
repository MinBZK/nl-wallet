pub mod data_uri;
pub mod error;
#[cfg(feature = "server")]
pub mod health;
#[cfg(feature = "client")]
pub mod reqwest;
pub mod tls;
pub mod urls;
