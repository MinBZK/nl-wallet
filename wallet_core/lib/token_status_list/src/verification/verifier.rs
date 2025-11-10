use chrono::DateTime;
use chrono::Utc;
use log::warn;
use rustls_pki_types::TrustAnchor;

use crypto::x509::BorrowingCertificate;
use http_utils::urls::HttpsUri;
use utils::generator::Generator;

use crate::status_list::StatusType;
use crate::verification::client::StatusListClient;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RevocationStatus {
    Valid,
    Invalid,
    Undetermined,
    Corrupted,
}

pub struct RevocationVerifier<C>(C);

impl<C> RevocationVerifier<C>
where
    C: StatusListClient,
{
    pub fn new(client: C) -> Self {
        Self(client)
    }

    pub async fn verify(
        &self,
        issuer_trust_anchors: &[TrustAnchor<'_>],
        attestation_signing_certificate: &BorrowingCertificate,
        uri: &HttpsUri,
        time: &impl Generator<DateTime<Utc>>,
        index: usize,
    ) -> RevocationStatus {
        match self.0.fetch(uri).await {
            Ok(status_list_token) => match status_list_token.parse_and_verify(
                issuer_trust_anchors,
                attestation_signing_certificate,
                uri,
                time,
            ) {
                Ok(status_list) => match status_list.get(index) {
                    StatusType::Valid => RevocationStatus::Valid,
                    _ => RevocationStatus::Invalid,
                },
                Err(err) => {
                    warn!("Status list token fails verification: {err}");
                    RevocationStatus::Corrupted
                }
            },
            Err(err) => {
                warn!("Status list token fetching fails: {err}");
                RevocationStatus::Undetermined
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::ops::Add;

    use chrono::Days;
    use chrono::Utc;
    use futures::FutureExt;

    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use http_utils::urls::HttpsUri;
    use jwt::error::JwtError;
    use utils::generator::mock::MockTimeGenerator;

    use crate::status_list_token::StatusListToken;
    use crate::status_list_token::mock::create_status_list_token;
    use crate::verification::client::MockStatusListClient;
    use crate::verification::client::StatusListClient;
    use crate::verification::client::StatusListClientError;
    use crate::verification::verifier::RevocationStatus;
    use crate::verification::verifier::RevocationVerifier;

    struct StatusListClientStub(KeyPair);

    impl StatusListClient for StatusListClientStub {
        async fn fetch(&self, _uri: &HttpsUri) -> Result<StatusListToken, StatusListClientError> {
            let (_, _, status_list_token) =
                create_status_list_token(&self.0, Utc::now().add(Days::new(1)).timestamp()).await;

            Ok(status_list_token)
        }
    }

    #[test]
    fn test_verify() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();
        let iss_keypair = ca.generate_issuer_mock().unwrap();

        let client = StatusListClientStub(keypair);

        let verifier = RevocationVerifier::new(client);

        // Index 1 is valid
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate(),
                &"https://example.com/statuslists/1".parse().unwrap(),
                &MockTimeGenerator::default(),
                1,
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Valid, status);

        // Index 3 is invalid
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate(),
                &"https://example.com/statuslists/1".parse().unwrap(),
                &MockTimeGenerator::default(),
                3,
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Invalid, status);

        // Corrupted when the sub claim doesn't match
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate(),
                &"https://different_uri".parse().unwrap(),
                &MockTimeGenerator::default(),
                1,
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Corrupted when the JWT doesn't validate
        let status = verifier
            .verify(
                &[],
                iss_keypair.certificate(),
                &"https://example.com/statuslists/1".parse().unwrap(),
                &MockTimeGenerator::default(),
                1,
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Corrupted when the JWT is expired
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate(),
                &"https://example.com/statuslists/1".parse().unwrap(),
                &MockTimeGenerator::new(Utc::now().add(Days::new(2))),
                1,
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Corrupted when the attestation is signed with a different certificate
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                ca.generate_pid_issuer_mock().unwrap().certificate(),
                &"https://example.com/statuslists/1".parse().unwrap(),
                &MockTimeGenerator::default(),
                1,
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Undetermined when retrieving the status list fails
        let mut client = MockStatusListClient::new();
        client
            .expect_fetch()
            .returning(|_| Err(StatusListClientError::JwtParsing(JwtError::MissingX5c)));
        let verifier = RevocationVerifier::new(client);
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate(),
                &"https://example.com/statuslists/1".parse().unwrap(),
                &MockTimeGenerator::default(),
                1,
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Undetermined, status);
    }
}
