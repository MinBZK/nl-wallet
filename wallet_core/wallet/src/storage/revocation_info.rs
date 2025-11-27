use chrono::DateTime;
use chrono::Utc;
use rustls_pki_types::TrustAnchor;
use uuid::Uuid;

use attestation_types::status_claim::StatusListClaim;
use crypto::x509::DistinguishedName;
use entity::revocation_info;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::verifier::RevocationStatus;
use token_status_list::verification::verifier::RevocationVerifier;
use utils::generator::Generator;

/// An instance of an attestation copy's revocation information
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(derive_more::Constructor))]
pub struct RevocationInfo {
    pub(super) attestation_copy_id: Uuid,
    pub(super) status_list_claim: StatusListClaim,
    pub(super) issuer_cert_distinguished_name: DistinguishedName,
}

impl RevocationInfo {
    pub fn attestation_copy_id(&self) -> Uuid {
        self.attestation_copy_id
    }

    pub async fn verify_revocation(
        &self,
        issuer_trust_anchors: &[TrustAnchor<'_>],
        revocation_verifier: &RevocationVerifier<impl StatusListClient>,
        time: &impl Generator<DateTime<Utc>>,
    ) -> RevocationStatus {
        revocation_verifier
            .verify(
                issuer_trust_anchors,
                self.issuer_cert_distinguished_name.clone(),
                self.status_list_claim.uri.clone(),
                time,
                self.status_list_claim.idx.try_into().unwrap(),
            )
            .await
    }
}

impl From<revocation_info::RevocationInfo> for RevocationInfo {
    fn from(value: revocation_info::RevocationInfo) -> Self {
        RevocationInfo {
            attestation_copy_id: value.id,
            status_list_claim: StatusListClaim {
                uri: value.status_list_url.parse().expect("URL has been parsed before"),
                idx: value.status_list_index,
            },
            issuer_cert_distinguished_name: value.issuer_certificate_dn,
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use futures::FutureExt;
    use uuid::Uuid;

    use attestation_types::status_claim::StatusClaim;
    use crypto::server_keys::generate::Ca;
    use token_status_list::verification::client::mock::StatusListClientStub;
    use token_status_list::verification::verifier::RevocationStatus;
    use token_status_list::verification::verifier::RevocationVerifier;
    use utils::generator::mock::MockTimeGenerator;

    use crate::storage::revocation_info::RevocationInfo;

    #[test]
    fn test_verify_revocation() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_cert = ca.generate_status_list_mock().unwrap();
        let issuer_cert_dn = issuer_cert.certificate().distinguished_name_canonical().unwrap();
        let issuer_trust_anchors = &[ca.to_trust_anchor()];

        let revocation_verifier = RevocationVerifier::new(Arc::new(StatusListClientStub::new(issuer_cert)));

        let StatusClaim::StatusList(claim) = StatusClaim::new_mock();

        let revocation_info = RevocationInfo::new(Uuid::new_v4(), claim, issuer_cert_dn);

        let status = revocation_info
            .verify_revocation(
                issuer_trust_anchors,
                &revocation_verifier,
                &MockTimeGenerator::default(),
            )
            .now_or_never()
            .unwrap();

        assert_matches!(status, RevocationStatus::Valid);
    }
}
