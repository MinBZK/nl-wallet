use std::{error::Error, time::Duration};

use base64::prelude::*;
use reqwest::{Certificate, Client, ClientBuilder};
use serde::{Deserialize, Deserializer};

const CLIENT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const CLIENT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

fn base64_to_certificate(encoded_certificate: String) -> Result<reqwest::Certificate, Box<dyn Error>> {
    let der_bytes = BASE64_STANDARD.decode(encoded_certificate)?;
    let certificate = reqwest::Certificate::from_der(&der_bytes)?;
    Ok(certificate)
}

pub fn deserialize_certificate<'de, D>(deserializer: D) -> Result<reqwest::Certificate, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded_certificate = String::deserialize(deserializer).map_err(serde::de::Error::custom)?;
    let certificate = base64_to_certificate(encoded_certificate).map_err(serde::de::Error::custom)?;
    Ok(certificate)
}

pub fn deserialize_certificates<'de, D>(deserializer: D) -> Result<Vec<reqwest::Certificate>, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded_certificates: Vec<String> = Vec::deserialize(deserializer).map_err(serde::de::Error::custom)?;
    let certificates = encoded_certificates
        .into_iter()
        .map(|encoded_certificate| {
            let certificate = base64_to_certificate(encoded_certificate).map_err(serde::de::Error::custom)?;
            Ok(certificate)
        })
        .collect::<Result<_, _>>()?;

    Ok(certificates)
}

pub fn default_reqwest_client_builder() -> ClientBuilder {
    Client::builder()
        .timeout(CLIENT_REQUEST_TIMEOUT)
        .connect_timeout(CLIENT_CONNECT_TIMEOUT)
}

/// Create a [`ClientBuilder`] that validates certificates signed with the supplied trust anchors (root certificates) as
/// well as the built-in root certificates.
pub fn trusted_reqwest_client_builder(trust_anchors: Vec<Certificate>) -> ClientBuilder {
    trust_anchors.into_iter().fold(
        default_reqwest_client_builder().tls_built_in_root_certs(true),
        |builder, root_ca| builder.add_root_certificate(root_ca),
    )
}

/// Create a [`ClientBuilder`] that only validates certificates signed with the supplied trust anchors (root certificates).
/// The built-in root certificates are therefore disabled and the client will only work over https.
pub fn tls_pinned_client_builder(trust_anchors: Vec<Certificate>) -> ClientBuilder {
    trusted_reqwest_client_builder(trust_anchors)
        .https_only(true)
        .tls_built_in_root_certs(false)
}
