use std::error::Error;

use jwt::nonce::Nonce;
use jwt::wia::WiaDisclosure;

pub trait WiaClient {
    type Error: Error + Send + Sync + 'static;

    async fn issue_wia(&self, aud: String, nonce: Option<Nonce>) -> Result<WiaDisclosure, Self::Error>;
}

#[cfg(feature = "mock")]
pub mod mock {
    use std::convert::Infallible;
    use std::time::Duration;

    use attestation_types::status_claim::StatusClaim;
    use chrono::Utc;
    use crypto::PublicKey;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::p256_der::verifying_key_sha256;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use futures::FutureExt;
    use jwt::SignedJwt;
    use jwt::nonce::Nonce;
    use jwt::wia::ClientStatus;
    use jwt::wia::WiaClaims;
    use jwt::wia::WiaDisclosure;
    use jwt::wia::WiaPopClaims;
    use jwt::wia::WiaWalletInfo;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::Generate;
    use utils::generator::mock::MockTimeGenerator;

    use super::WiaClient;
    use crate::mock::MOCK_WALLET_CLIENT_ID;

    #[derive(Debug, Default)]
    pub struct MockWiaClient {
        wia_keypair: Option<KeyPair>,
        client_id: Option<String>,
    }

    impl MockWiaClient {
        pub fn new() -> Self {
            Self {
                wia_keypair: None,
                client_id: None,
            }
        }

        pub fn new_with_wia_keypair(wia_keypair: KeyPair) -> Self {
            Self {
                wia_keypair: Some(wia_keypair),
                client_id: None,
            }
        }

        /// Issue a WIA whose `sub` (and PoP `iss`) claim is the given `client_id`, instead of the default
        /// [`MOCK_WALLET_CLIENT_ID`].
        pub fn new_with_client_id(wia_keypair: KeyPair, client_id: String) -> Self {
            Self {
                wia_keypair: Some(wia_keypair),
                client_id: Some(client_id),
            }
        }
    }

    impl WiaClient for MockWiaClient {
        type Error = Infallible;

        async fn issue_wia(&self, aud: String, challenge: Option<Nonce>) -> Result<WiaDisclosure, Self::Error> {
            let wia_key = SigningKey::generate();
            let wia_key = MockRemoteEcdsaKey::new(verifying_key_sha256(wia_key.verifying_key()), wia_key);

            let wia_keypair = self
                .wia_keypair
                .clone()
                .unwrap_or_else(|| Ca::generate_issuer_mock_ca().unwrap().generate_wia_mock().unwrap());

            let client_id = self
                .client_id
                .clone()
                .unwrap_or_else(|| MOCK_WALLET_CLIENT_ID.to_string());

            let exp = Utc::now() + Duration::from_secs(600);

            let time = MockTimeGenerator::default();
            let wia = SignedJwt::sign_with_iat(
                &WiaClaims::new(
                    &PublicKey::from(*wia_key.verifying_key()),
                    wia_keypair.certificate().common_name().unwrap().unwrap().to_string(),
                    client_id.clone(),
                    exp.into(),
                    WiaWalletInfo::new_mock(),
                    ClientStatus {
                        status: StatusClaim::new_mock(),
                        exp: exp.into(),
                    },
                    &time,
                )
                .unwrap(),
                &wia_keypair,
                &time,
            )
            .now_or_never()
            .unwrap()
            .unwrap()
            .into();

            let wia_disclosure = SignedJwt::sign(
                &WiaPopClaims {
                    iss: client_id,
                    aud,
                    iat: Utc::now().into(),
                    jti: "jti".to_string(),
                    challenge,
                },
                &wia_key,
            )
            .now_or_never()
            .unwrap()
            .unwrap();

            Ok(WiaDisclosure::new(wia, wia_disclosure.into()))
        }
    }
}
