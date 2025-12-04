use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use tracing::warn;

use attestation_types::status_claim::StatusClaim;
use attestation_types::status_claim::StatusClaim::StatusList;
use attestation_types::status_claim::StatusListClaim;
use crypto::x509::DistinguishedName;
use utils::generator::Generator;

use crate::status_list::StatusType;
use crate::verification::client::StatusListClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum RevocationStatus {
    Valid,
    Invalid,
    Undetermined,
    Corrupted,
}

#[derive(Debug)]
pub struct RevocationVerifier<C>(Arc<C>);

impl<C> RevocationVerifier<C>
where
    C: StatusListClient,
{
    pub fn new(client: Arc<C>) -> Self {
        Self(client)
    }

    pub async fn verify(
        &self,
        issuer_trust_anchors: &[TrustAnchor<'_>],
        attestation_signing_certificate_dn: DistinguishedName,
        status_claim: StatusClaim,
        time: &impl Generator<DateTime<Utc>>,
    ) -> RevocationStatus {
        let StatusList(StatusListClaim { uri, idx }) = status_claim;

        let status_list_token = match self.0.fetch(uri.clone()).await {
            Ok(token) => token,
            Err(err) => {
                warn!("Status list token fetching fails: {err}");
                return RevocationStatus::Undetermined;
            }
        };

        match status_list_token.parse_and_verify(issuer_trust_anchors, attestation_signing_certificate_dn, &uri, time) {
            Ok(status_list) => match status_list.single_unpack(idx.try_into().unwrap()) {
                StatusType::Valid => RevocationStatus::Valid,
                _ => RevocationStatus::Invalid,
            },
            Err(err) => {
                warn!("Status list token fails verification: {err}");
                RevocationStatus::Corrupted
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::ops::Add;
    use std::sync::Arc;

    use chrono::Days;
    use chrono::Utc;
    use futures::FutureExt;

    use attestation_types::status_claim::StatusClaim::StatusList;
    use attestation_types::status_claim::StatusListClaim;
    use crypto::server_keys::generate::Ca;
    use crypto::x509::DistinguishedName;
    use jwt::error::JwtError;
    use utils::generator::mock::MockTimeGenerator;

    use crate::verification::client::StatusListClientError;
    use crate::verification::client::mock::MockStatusListClient;
    use crate::verification::client::mock::StatusListClientStub;
    use crate::verification::verifier::RevocationStatus;
    use crate::verification::verifier::RevocationVerifier;

    #[test]
    fn test_verify() {
        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();
        let iss_keypair = ca.generate_issuer_mock().unwrap();

        let verifier = RevocationVerifier::new(Arc::new(StatusListClientStub::new(keypair)));

        // Index 1 is valid
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Valid, status);

        // Index 3 is invalid
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 3,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Invalid, status);

        // Corrupted when the sub claim doesn't match
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
                StatusList(StatusListClaim {
                    uri: "https://different_uri".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Corrupted when the JWT doesn't validate
        let status = verifier
            .verify(
                &[],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Corrupted when the JWT is expired
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::new(Utc::now().add(Days::new(2))),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Corrupted when the attestation is signed with a different certificate
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                DistinguishedName::new(String::from("CN=Different CA")),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Corrupted, status);

        // Undetermined when retrieving the status list fails
        let mut client = MockStatusListClient::new();
        client
            .expect_fetch()
            .returning(|_| Err(StatusListClientError::JwtParsing(JwtError::MissingX5c.into())));
        let verifier = RevocationVerifier::new(Arc::new(client));
        let status = verifier
            .verify(
                &[ca.to_trust_anchor()],
                iss_keypair.certificate().distinguished_name_canonical().unwrap(),
                StatusList(StatusListClaim {
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                    idx: 1,
                }),
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();
        assert_eq!(RevocationStatus::Undetermined, status);
    }
}
