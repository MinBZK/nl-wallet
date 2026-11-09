use url::Url;

use crate::status_list_token::StatusListFormat;
use crate::status_list_token::StatusListToken;
use crate::status_list_token::StatusListTokenParseError;

#[derive(Debug, thiserror::Error)]
pub enum StatusListClientError {
    #[error("networking error: {0}")]
    Networking(#[from] reqwest::Error),

    #[error("status list response has unexpected content type: {0}")]
    UnexpectedContentType(String),

    #[error("could not parse status list response: {0}")]
    Parse(#[source] StatusListTokenParseError),
}

#[trait_variant::make(Send)]
pub trait StatusListClient {
    async fn fetch(&self, url: Url, format: StatusListFormat) -> Result<StatusListToken, StatusListClientError>;
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::collections::HashMap;
    use std::ops::Add;
    use std::time::Duration;

    use chrono::Days;
    use chrono::Utc;
    use crypto::keys::EcdsaKeySend;
    use crypto::server_keys::KeyPair;
    use derive_more::Constructor;
    use url::Url;
    use utils::generator::TimeGenerator;

    use crate::status_list::StatusList;
    use crate::status_list_cwt::StatusListCwt;
    use crate::status_list_jwt::StatusListJwt;
    use crate::status_list_jwt::mock::mock_status_list_jwt;
    use crate::status_list_token::StatusListFormat;
    use crate::status_list_token::StatusListToken;
    use crate::verification::client::StatusListClient;
    use crate::verification::client::StatusListClientError;

    mockall::mock! {
        pub StatusListClient {}

        impl StatusListClient for StatusListClient {
            async fn fetch(&self, url: Url, format: StatusListFormat) -> Result<StatusListToken, StatusListClientError>;
        }
    }

    #[derive(Debug, Constructor)]
    pub struct StatusListClientStub<S>(KeyPair<S>);

    impl<S> StatusListClient for StatusListClientStub<S>
    where
        S: EcdsaKeySend + Sync,
    {
        async fn fetch(&self, _url: Url, _format: StatusListFormat) -> Result<StatusListToken, StatusListClientError> {
            let (_, _, status_list_token) =
                mock_status_list_jwt(&self.0, Some(Utc::now().add(Days::new(1)).timestamp()), None).await;

            Ok(status_list_token.into())
        }
    }

    // This is the client that belongs to the [`status_list_service::MockStatusListServices`] struct.
    // It contains a map of keypairs by attestation_group, where the attestation_group that should be
    // used for the lookup is extracted from the `url` parameter in the same manner as the
    // [`MockStatusListServices`].
    #[derive(Debug, Constructor)]
    pub struct MockStatusListServicesClient<S>(HashMap<String, KeyPair<S>>);

    impl<S> StatusListClient for MockStatusListServicesClient<S>
    where
        S: EcdsaKeySend + Sync,
    {
        async fn fetch(&self, url: Url, format: StatusListFormat) -> Result<StatusListToken, StatusListClientError> {
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

            let claims = StatusListToken::builder(url, StatusList::new(10).pack())
                .ttl(Some(Duration::from_secs(3600)))
                .build(&TimeGenerator);
            let status_list_token = match format {
                StatusListFormat::Jwt => StatusListJwt::sign(claims, keypair).await.unwrap().into(),
                StatusListFormat::Cwt => StatusListCwt::sign(claims, keypair).await.unwrap().into(),
            };

            Ok(status_list_token)
        }
    }
}
