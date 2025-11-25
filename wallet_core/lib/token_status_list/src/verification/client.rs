use std::sync::Arc;

use url::Url;

use jwt::error::JwtError;

use crate::status_list_token::StatusListToken;

#[derive(Debug, Clone, thiserror::Error)]
pub enum StatusListClientError {
    #[error("networking error: {0}")]
    Networking(#[from] Arc<reqwest::Error>),

    #[error("jwt parsing error: {0}")]
    JwtParsing(#[from] Arc<JwtError>),
}

#[trait_variant::make(Send)]
pub trait StatusListClient {
    async fn fetch(&self, url: Url) -> Result<StatusListToken, StatusListClientError>;
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::collections::HashMap;
    use std::ops::Add;
    use std::sync::Arc;
    use std::time::Duration;

    use chrono::Days;
    use chrono::Utc;
    use derive_more::Constructor;
    use url::Url;

    use crypto::EcdsaKeySend;
    use crypto::server_keys::KeyPair;

    use crate::status_list::StatusList;
    use crate::status_list_token::StatusListToken;
    use crate::status_list_token::mock::create_status_list_token;
    use crate::verification::client::StatusListClient;
    use crate::verification::client::StatusListClientError;

    mockall::mock! {
        pub StatusListClient {}

        impl StatusListClient for StatusListClient {
            async fn fetch(&self, url: Url) -> Result<StatusListToken, StatusListClientError>;
        }
    }

    #[derive(Debug, Constructor)]
    pub struct StatusListClientStub<S>(KeyPair<S>);

    impl<S> StatusListClient for StatusListClientStub<S>
    where
        S: EcdsaKeySend + Sync,
    {
        async fn fetch(&self, _url: Url) -> Result<StatusListToken, StatusListClientError> {
            let (_, _, status_list_token) =
                create_status_list_token(&self.0, Some(Utc::now().add(Days::new(1)).timestamp()), None).await;

            Ok(status_list_token)
        }
    }

    // This is the client that belongs to the [`status_list_service::MockStatusListServices`] struct.
    // It contains a map of keypairs by attestation_type, where the attestation_type that should be
    // used for the lookup is extracted from the `url` parameter in the same manner as the
    // [`MockStatusListServices`].
    #[derive(Debug, Constructor)]
    pub struct MockStatusListServicesClient<S>(HashMap<String, KeyPair<S>>);

    impl<S> StatusListClient for MockStatusListServicesClient<S>
    where
        S: EcdsaKeySend + Sync,
    {
        async fn fetch(&self, url: Url) -> Result<StatusListToken, StatusListClientError> {
            let url_vct = url
                .as_str()
                .rsplit("/")
                .next()
                .expect("url should always have at least one path segment");

            let (_, keypair) = self
                .0
                .iter()
                .find(|(vct, _)| vct.replace(':', "-") == url_vct)
                .expect("could not find keypair for url for signing status list token");

            let status_list_token = StatusListToken::builder(url, StatusList::new(10).pack())
                .ttl(Some(Duration::from_secs(3600)))
                .sign(keypair)
                .await
                .map_err(Arc::new)?;

            Ok(status_list_token)
        }
    }
}
