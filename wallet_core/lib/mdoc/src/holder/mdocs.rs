use std::result::Result;

use chrono::DateTime;
use chrono::Utc;
use rustls_pki_types::TrustAnchor;
use ssri::Integrity;

use crypto::keys::CredentialEcdsaKey;
use crypto::x509::BorrowingCertificate;
use utils::generator::Generator;

use crate::errors::Error;
use crate::iso::*;
use crate::utils::cose::CoseError;
use crate::utils::serialization::TaggedBytes;
use crate::verifier::IssuerSignedVerificationResult;
use crate::verifier::ValidityRequirement;

use super::HolderError;

/// A full mdoc: everything needed to disclose attributes from the mdoc.
#[derive(Debug, Clone, PartialEq)]
pub struct Mdoc {
    /// Mobile Security Object of the mdoc. This is also present inside the `issuer_signed`; we include it here for
    /// convenience (fetching it from the `issuer_signed` would involve parsing the COSE inside it).
    mso: MobileSecurityObject,

    /// Identifier of the mdoc's private key. Obtain a reference to it with
    /// [`DisclosureWscd::new_key(private_key_id, public_key)`].
    // TODO (PVW-4962): Move this field to the `wallet` crate, as it is a concern of `Wallet`.
    private_key_id: String,
    issuer_signed: IssuerSigned,
}

impl Mdoc {
    /// Construct a new `Mdoc`, verifying it against the specified thrust anchors before returning it.
    pub fn new<K: CredentialEcdsaKey>(
        private_key_id: String,
        issuer_signed: IssuerSigned,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> crate::Result<Mdoc> {
        // Unfortunately we have to discard the `attributes` here, even though
        // the only consumer of this function will need them right away.
        let IssuerSignedVerificationResult { mso, .. } =
            issuer_signed.verify(ValidityRequirement::AllowNotYetValid, time, trust_anchors)?;

        let mdoc = Mdoc {
            mso,
            private_key_id,
            issuer_signed,
        };

        Ok(mdoc)
    }

    /// Construct a new `Mdoc` by parsing the `issuer_auth` field of an `IssuerSigned` without validating it.
    pub fn dangerous_parse_unverified(issuer_signed: IssuerSigned, private_key_id: String) -> Result<Self, CoseError> {
        let TaggedBytes(mso) = issuer_signed.issuer_auth.dangerous_parse_unverified()?;

        let mdoc = Self {
            mso,
            private_key_id,
            issuer_signed,
        };

        Ok(mdoc)
    }

    pub fn doc_type(&self) -> &str {
        &self.mso.doc_type
    }

    pub fn issuer_signed(&self) -> &IssuerSigned {
        &self.issuer_signed
    }

    pub fn into_issuer_signed(self) -> IssuerSigned {
        self.issuer_signed
    }

    pub fn private_key_id(&self) -> &str {
        &self.private_key_id
    }

    pub fn into_private_key_id(self) -> String {
        self.private_key_id
    }

    pub fn into_components(self) -> (MobileSecurityObject, String, IssuerSigned) {
        let Self {
            mso,
            private_key_id,
            issuer_signed,
        } = self;

        (mso, private_key_id, issuer_signed)
    }

    pub fn issuer_certificate(&self) -> Result<BorrowingCertificate, CoseError> {
        self.issuer_signed.issuer_auth.signing_cert()
    }

    pub fn type_metadata_integrity(&self) -> Result<&Integrity, Error> {
        let integrity = self
            .mso
            .type_metadata_integrity
            .as_ref()
            .ok_or(HolderError::MissingMetadataIntegrity)?;

        Ok(integrity)
    }
}

#[cfg(any(test, feature = "test"))]
mod test {
    use chrono::DateTime;
    use chrono::TimeDelta;
    use chrono::Utc;
    use futures::FutureExt;
    use indexmap::IndexMap;
    use ssri::Integrity;

    use crypto::CredentialEcdsaKey;
    use crypto::server_keys::generate::Ca;
    use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
    use crypto::x509::CertificateUsage;
    use http_utils::urls::HttpsUri;
    use token_status_list::status_claim::StatusClaim;
    use utils::generator::Generator;

    use crate::iso::disclosure::IssuerSigned;
    use crate::iso::mdocs::DigestAlgorithm;
    use crate::iso::mdocs::Entry;
    use crate::iso::mdocs::IssuerNameSpaces;
    use crate::iso::mdocs::IssuerSignedItemBytes;
    use crate::iso::mdocs::MobileSecurityObject;
    use crate::iso::mdocs::MobileSecurityObjectVersion;
    use crate::iso::mdocs::ValidityInfo;
    use crate::utils::cose::CoseKey;
    use crate::utils::cose::MdocCose;
    use crate::utils::serialization::TaggedBytes;

    use super::Mdoc;

    impl Mdoc {
        pub fn new_unverified(mso: MobileSecurityObject, private_key_id: String, issuer_signed: IssuerSigned) -> Self {
            Self {
                mso,
                private_key_id,
                issuer_signed,
            }
        }

        #[expect(clippy::too_many_arguments)]
        pub async fn new_unverified_from_data(
            doc_type: String,
            issuer_uri: HttpsUri,
            name_spaces: IndexMap<String, Vec<Entry>>,
            metadata_integrity: Integrity,
            ca: &Ca,
            device_key: &impl CredentialEcdsaKey,
            time_generator: &impl Generator<DateTime<Utc>>,
        ) -> Self {
            let time = time_generator.generate();

            let issuer_key_pair = ca
                .generate_key_pair(ISSUANCE_CERT_CN, CertificateUsage::Mdl, Default::default())
                .unwrap();

            let device_public_key = &device_key.verifying_key().await.unwrap();
            let cose_pubkey = CoseKey::try_from(device_public_key).unwrap();

            let name_spaces = IssuerNameSpaces::try_from(name_spaces).unwrap();

            let mso = MobileSecurityObject {
                version: MobileSecurityObjectVersion::V1_0,
                digest_algorithm: DigestAlgorithm::SHA256,
                doc_type,
                value_digests: (&name_spaces).try_into().unwrap(),
                device_key_info: cose_pubkey.into(),
                validity_info: ValidityInfo {
                    signed: time.into(),
                    valid_from: time.into(),
                    valid_until: (time + TimeDelta::days(365)).into(),
                    expected_update: None,
                },
                issuer_uri: Some(issuer_uri),
                attestation_qualification: Some(Default::default()),
                status: Some(StatusClaim::new_mock()),
                type_metadata_integrity: Some(metadata_integrity),
            };

            let header = IssuerSigned::create_unprotected_header(issuer_key_pair.certificate().to_vec());
            let mso_tagged = TaggedBytes(mso);
            let issuer_auth = MdocCose::sign(&mso_tagged, header, &issuer_key_pair, true)
                .now_or_never()
                .unwrap()
                .unwrap();

            let TaggedBytes(mso) = mso_tagged;
            let private_key_id = device_key.identifier().to_string();
            let issuer_signed = IssuerSigned {
                name_spaces: Some(name_spaces),
                issuer_auth,
            };

            Self::new_unverified(mso, private_key_id, issuer_signed)
        }

        pub fn modify_attributes<F>(&mut self, name_space: &str, modify_func: F)
        where
            F: FnOnce(&mut Vec<IssuerSignedItemBytes>),
        {
            let name_spaces = self.issuer_signed.name_spaces.as_mut().unwrap();
            name_spaces.modify_attributes(name_space, modify_func);
        }
    }
}

#[cfg(any(test, feature = "mock_example_constructors"))]
pub mod mock {
    use chrono::DateTime;
    use chrono::Utc;
    use ciborium::Value;
    use indexmap::IndexMap;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use crypto::CredentialEcdsaKey;
    use crypto::examples::EXAMPLE_KEY_IDENTIFIER;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::generate::Ca;
    use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use utils::generator::Generator;
    use utils::generator::mock::MockTimeGenerator;

    use crate::examples::IsoCertTimeGenerator;
    use crate::iso::disclosure::DeviceResponse;
    use crate::iso::mdocs::Entry;

    use super::Mdoc;

    impl Mdoc {
        /// Out of the example data structures in the standard, assemble an mdoc.
        /// The issuer-signed part of the mdoc is based on a [`DeviceResponse`] in which not all attributes of the
        /// originating mdoc are disclosed. Consequentially, the issuer signed-part of the resulting mdoc misses some
        /// [`IssuerSignedItem`] instances, i.e. attributes.
        /// This is because the other attributes are actually nowhere present in the standard so it is impossible to
        /// construct the example mdoc with all attributes present.
        ///
        /// Using tests should not rely on all attributes being present.
        pub async fn new_example_resigned(ca: &Ca) -> Self {
            let issuer_signed = DeviceResponse::example_resigned(ca)
                .await
                .documents
                .unwrap()
                .into_iter()
                .next()
                .unwrap()
                .issuer_signed;

            Mdoc::new::<MockRemoteEcdsaKey>(
                EXAMPLE_KEY_IDENTIFIER.to_string(),
                issuer_signed,
                &IsoCertTimeGenerator,
                &[ca.to_trust_anchor()],
            )
            .unwrap()
        }

        pub async fn new_mock() -> Self {
            let ca = Ca::generate_issuer_mock_ca().unwrap();
            let key = MockRemoteEcdsaKey::new("identifier".to_owned(), SigningKey::random(&mut OsRng));
            Self::new_mock_with_ca_and_key(&ca, &key).await
        }

        pub async fn new_mock_with_ca_and_key(ca: &Ca, device_key: &MockRemoteEcdsaKey) -> Self {
            Self::new_unverified_nl_pid_example(ca, device_key, &MockTimeGenerator::default()).await
        }

        pub async fn new_unverified_nl_pid_example(
            ca: &Ca,
            device_key: &impl CredentialEcdsaKey,
            time_generator: &impl Generator<DateTime<Utc>>,
        ) -> Self {
            let (metadata_integrity, _) = TypeMetadataDocuments::nl_pid_example();

            Self::new_unverified_from_data(
                PID_ATTESTATION_TYPE.to_string(),
                format!("https://{ISSUANCE_CERT_CN}").parse().unwrap(),
                IndexMap::from_iter(vec![(
                    PID_ATTESTATION_TYPE.to_string(),
                    vec![
                        Entry {
                            name: "bsn".to_string(),
                            value: Value::Text("999999999".to_string()),
                        },
                        Entry {
                            name: "given_name".to_string(),
                            value: Value::Text("Willeke Liselotte".to_string()),
                        },
                        Entry {
                            name: "family_name".to_string(),
                            value: Value::Text("De Bruijn".to_string()),
                        },
                    ],
                )]),
                metadata_integrity,
                ca,
                device_key,
                time_generator,
            )
            .await
        }
    }
}
