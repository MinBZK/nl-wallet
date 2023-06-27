use anyhow::Result;
use async_trait::async_trait;
use axum_test_helper::TestClient;

use once_cell::sync::Lazy;
use url::Url;

use platform_support::hw_keystore::software::SoftwareEcdsaKey;
use wallet::{
    mock::{MockConfigurationRepository, MockStorage},
    wallet::{AccountServerClient, AccountServerClientError, Wallet},
};
use wallet_common::account::{
    auth::{Certificate, Challenge, Registration, WalletCertificate},
    jwt::EcdsaDecodingKey,
    signed::SignedDouble,
};

use wallet_provider::{account_server::stub::account_server, app};

// The global test client and account server public key.
static TEST_CLIENT: Lazy<(TestClient, EcdsaDecodingKey)> = Lazy::new(|| {
    let account_server = account_server();
    let account_server_pubkey = account_server.pubkey.clone();
    let app = app::router(account_server);
    let client = TestClient::new(app);

    (client, account_server_pubkey)
});

/// This struct acts as a client for [`Wallet`] by implementing [`AccountServerClient`]
/// and using [`TestClient`]. It can access the routes of the Wallet Provider without
/// actually needing a HTTP server.
struct WalletTestClient {
    client: &'static TestClient,
}

#[async_trait]
impl AccountServerClient for WalletTestClient {
    fn new(_base_url: &Url) -> Self
    where
        Self: Sized,
    {
        WalletTestClient { client: &TEST_CLIENT.0 }
    }

    async fn registration_challenge(&self) -> Result<Vec<u8>, AccountServerClientError> {
        let challenge = self
            .client
            .post("/api/v1/enroll")
            .send()
            .await
            .json::<Challenge>()
            .await
            .challenge
            .0;

        Ok(challenge)
    }

    async fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, AccountServerClientError> {
        let cert = self
            .client
            .post("/api/v1/createwallet")
            .json(&registration_message)
            .send()
            .await
            .json::<Certificate>()
            .await
            .certificate;

        Ok(cert)
    }
}

/// Create an instance of [`Wallet`] with appropriate mocks, including [`WalletTestClient`].
async fn create_test_wallet() -> Wallet<MockConfigurationRepository, WalletTestClient, MockStorage, SoftwareEcdsaKey> {
    let mut config = MockConfigurationRepository::default();
    config.0.account_server.public_key = TEST_CLIENT.1.clone();

    Wallet::new(config).await.expect("Could not create test wallet")
}

#[tokio::test]
async fn test_wallet_registration() {
    let mut wallet = create_test_wallet().await;

    // No registration should be loaded initially.
    assert!(!wallet.has_registration());

    // Register with a valid PIN.
    wallet
        .register("112233".to_string())
        .await
        .expect("Could not register wallet");

    // The registration should now be loaded.
    assert!(wallet.has_registration());

    // TODO: check the contents of the mocked account server
    //       storage, once that feature is added.
}
