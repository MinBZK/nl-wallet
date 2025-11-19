use url::Url;

use jwt::error::JwtError;

use crate::status_list_token::StatusListToken;

#[derive(Debug, thiserror::Error)]
pub enum StatusListClientError {
    #[error("networking error: {0}")]
    Networking(#[from] reqwest::Error),

    #[error("jwt parsing error: {0}")]
    JwtParsing(#[from] JwtError),
}

#[trait_variant::make(Send)]
pub trait StatusListClient {
    async fn fetch(&self, url: Url) -> Result<StatusListToken, StatusListClientError>;
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::collections::HashMap;
    use std::ops::Add;
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
                create_status_list_token(&self.0, Utc::now().add(Days::new(1)).timestamp()).await;

            Ok(status_list_token)
        }
    }

    #[derive(Debug, Constructor)]
    pub struct MockStatusListServiceClient<S>(HashMap<String, KeyPair<S>>);

    impl<S> StatusListClient for MockStatusListServiceClient<S>
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
                .await?;

            Ok(status_list_token)
        }
    }
}
