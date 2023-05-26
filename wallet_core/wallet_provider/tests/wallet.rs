use anyhow::Result;
use async_trait::async_trait;
use axum_test_helper::TestClient;

use platform_support::hw_keystore::software::SoftwareEcdsaKey;
use wallet::{
    mock::MockStorage,
    wallet::{AccountServerClient, Wallet},
};
use wallet_common::account::{
    auth::{Certificate, Challenge, Registration, WalletCertificate},
    signed::SignedDouble,
};

use wallet_provider::{account_server::stub::account_server, app};

/// This struct acts as a client for [`Wallet`] by implementing [`AccountServerClient`]
/// and using [`TestClient`]. It can access the routes of the Wallet Provider without
/// actually needing a HTTP server.
struct WalletTestClient {
    client: TestClient,
}

impl WalletTestClient {
    fn new(client: TestClient) -> Self {
        WalletTestClient { client }
    }
}

#[async_trait]
impl AccountServerClient for WalletTestClient {
    type Error = std::convert::Infallible;

    async fn registration_challenge(&self) -> Result<Vec<u8>, Self::Error> {
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
    ) -> Result<WalletCertificate, Self::Error> {
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
async fn create_test_wallet() -> Wallet<WalletTestClient, MockStorage, SoftwareEcdsaKey> {
    let account_server = account_server();
    let pubkey = account_server.pubkey.clone();
    let app = app::router(account_server);
    let test_client = WalletTestClient::new(TestClient::new(app));
    let storage = MockStorage::default();

    Wallet::new(test_client, pubkey, storage)
        .await
        .expect("Could not create test wallet")
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
