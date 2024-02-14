use std::time::Duration;

use reqwest::{Certificate, Client, ClientBuilder};

const CLIENT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const CLIENT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

pub fn default_reqwest_client_builder() -> ClientBuilder {
    let client_builder = Client::builder();
    #[cfg(feature = "disable_tls_validation")]
    let client_builder = client_builder.danger_accept_invalid_certs(true);
    client_builder
        .timeout(CLIENT_REQUEST_TIMEOUT)
        .connect_timeout(CLIENT_CONNECT_TIMEOUT)
}

/// Create a [`ClientBuilder`] that only validates certificates signed with the supplied trust anchors (root certificates).
/// The built-in root certificates are therefore disabled and the client will only work over https.
pub fn tls_pinned_client_builder(trust_anchors: Vec<Certificate>) -> ClientBuilder {
    trust_anchors.into_iter().fold(
        default_reqwest_client_builder()
            .https_only(true)
            .tls_built_in_root_certs(false)
            .danger_accept_invalid_certs(false),
        |builder, root_ca| builder.add_root_certificate(root_ca),
    )
}
