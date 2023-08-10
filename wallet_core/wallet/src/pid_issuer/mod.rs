mod client;

use once_cell::sync::Lazy;

pub use client::PidIssuerClient;

/// Global variable to hold the `DigidClient` singleton.
pub static PID_ISSUER_CLIENT: Lazy<PidIssuerClient> = Lazy::new(PidIssuerClient::default);

#[derive(Debug, thiserror::Error)]
pub enum PidIssuerError {
    #[error("could not get BSN from PID issuer: {0}")]
    PidIssuer(#[from] reqwest::Error),
    #[error("could not get BSN from PID issuer: {0} - Response body: {1}")]
    PidIssuerResponse(#[source] reqwest::Error, String),
}
