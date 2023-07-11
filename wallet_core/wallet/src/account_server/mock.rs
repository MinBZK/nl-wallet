use async_trait::async_trait;
use url::Url;

use wallet_common::account::{
    messages::auth::{Registration, WalletCertificate},
    signed::SignedDouble,
};
use wallet_provider::{
    errors::{ConvertibleError, WalletProviderError},
    stub::{self, TestDeps},
    AccountServer,
};

use super::{AccountServerClient, AccountServerClientError, AccountServerResponseError};

impl AccountServerClientError {
    /// Helper method for converting account server errors directly to
    /// [`AccountServerResponseError`] instances. Unfortunately this cannot
    /// take the form of a `From<>` trait because it would conflict with `thiserror`.
    fn from_account_server<E>(value: E) -> Self
    where
        E: ConvertibleError,
    {
        let wp_error = WalletProviderError::from(value);

        AccountServerClientError::Response(AccountServerResponseError::Data(wp_error.status_code, wp_error.body))
    }
}

/// Mock implementation of [`AccountServerClient`] that is bound directly to
/// [`AccountServer`], skipping JSON encoding and HTTP(S).
#[async_trait]
impl AccountServerClient for AccountServer {
    fn new(_: &Url) -> Self
    where
        Self: Sized,
    {
        stub::account_server()
    }

    async fn registration_challenge(&self) -> Result<Vec<u8>, AccountServerClientError> {
        AccountServer::registration_challenge(self).map_err(AccountServerClientError::from_account_server)
    }

    async fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, AccountServerClientError> {
        AccountServer::register(self, &TestDeps, registration_message)
            .await
            .map_err(AccountServerClientError::from_account_server)
    }
}
