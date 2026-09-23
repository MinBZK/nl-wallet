use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Add;

use attestation_types::claim_path::ClaimPath;
use attestation_types::credential_format::Format;
use attestation_types::pid_constants::ADDRESS_ATTESTATION_TYPE;
use attestation_types::pid_constants::PID_ADDRESS_GROUP;
use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
use attestation_types::pid_constants::PID_BIRTH_DATE;
use attestation_types::pid_constants::PID_BSN;
use attestation_types::pid_constants::PID_FAMILY_NAME;
use attestation_types::pid_constants::PID_GIVEN_NAME;
use attestation_types::pid_constants::PID_RESIDENT_CITY;
use attestation_types::pid_constants::PID_RESIDENT_COUNTRY;
use attestation_types::pid_constants::PID_RESIDENT_HOUSE_NUMBER;
use attestation_types::pid_constants::PID_RESIDENT_POSTAL_CODE;
use attestation_types::pid_constants::PID_RESIDENT_STREET;
use attestation_types::status_claim::StatusClaim;
use chrono::Utc;
use crypto::PublicKey;
use crypto::mock_remote::MockRemoteWscd;
use crypto::server_keys::KeyPair;
use dcql::CredentialQueryIdentifier;
use dcql::Query;
use dcql::normalized::MdocAttributeRequest;
use dcql::normalized::NormalizedCredentialRequest;
use dcql::normalized::NormalizedCredentialRequests;
use dcql::normalized::SdJwtAttributeRequest;
use dcql::unique_id_vec::UniqueIdVec;
use derive_more::Debug;
use futures::FutureExt;
use indexmap::IndexMap;
use itertools::Itertools;
use mdoc::Entry;
use mdoc::holder::Mdoc;
use mdoc::holder::disclosure::PartialMdoc;
use sd_jwt::builder::SignedSdJwt;
use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use ssri::Integrity;
use utils::generator::mock::MockTimeGenerator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::attributes::Attribute;
use crate::attributes::Attributes;
use crate::attributes::AttributesTraversalBehaviour;
use crate::credential_payload::CredentialPayload;
use crate::credential_payload::PreviewableCredentialPayload;
use crate::disclosure::DisclosedAttestations;
use crate::disclosure::DisclosedAttributes;

/// A collection of [`TestCredential`] types, which are guaranteed to contain different credential query identifiers.
/// It provides aggregate versions of the methods present on that type. It also allows [`TestCredentials`] to be be
/// concatenated through the `+` operator.
#[derive(Debug, Clone)]
pub struct TestCredentials(VecNonEmpty<TestCredential>);

/// This type can be used when testing disclosure of credentials. It contains the following:
///
/// * A source credential.
/// * Type metadata for SD-JWT credentials. For mdoc disclosure, no metadata is needed.
/// * A subset of the the attributes of this credential, to be disclosed.
/// * An identifier for a generated credential query.
///
/// It provides methods to do the following:
///
/// * Generate the original credential in the form of either a `Mdoc` or `SdJwt` value.
/// * Generate a DCQL query for either format, normalized or otherwise.
/// * Generate the partial credential that is to be disclosed, in the form of either a `PartialMdoc` or
///   `UnsignedSdJwtPresentation`.
/// * Check the credential against a `DisclosedAttributes` value.
/// * Construct example values, based on the PID and address credential types.
#[derive(Debug, Clone)]
pub struct TestCredential {
    sd_jwt_payload_preview: PreviewableCredentialPayload,
    mdoc_attributes: Attributes,
    #[debug(skip)]
    type_metadata_documents: Option<TypeMetadataDocuments>,
    query_id: CredentialQueryIdentifier,
    sd_jwt_disclosure_attributes: Attributes,
    mdoc_disclosure_attributes: Attributes,
    status: StatusClaim,
}

impl TestCredentials {
    pub fn new(credentials: VecNonEmpty<TestCredential>) -> Self {
        let unique_id_count = credentials
            .iter()
            .map(|credential| &credential.query_id)
            .collect::<HashSet<_>>()
            .len();

        if unique_id_count < credentials.len().get() {
            panic!("test credential query identifiers should be unique");
        }

        Self(credentials)
    }

    pub fn to_normalized_credential_requests(
        &self,
        formats: impl IntoIterator<Item = Format>,
    ) -> NormalizedCredentialRequests {
        self.as_ref()
            .iter()
            .zip_eq(formats)
            .map(|(credential, format)| credential.to_normalized_credential_request(format))
            .collect_vec()
            .try_into()
            .unwrap()
    }

    pub fn to_dcql_query(&self, formats: impl IntoIterator<Item = Format>) -> Query {
        self.to_normalized_credential_requests(formats).into()
    }

    pub fn to_partial_mdocs(
        &self,
        issuer_keypair: &KeyPair,
        wscd: &impl AsRef<MockRemoteWscd>,
    ) -> HashMap<CredentialQueryIdentifier, VecNonEmpty<(PartialMdoc, String)>> {
        self.as_ref()
            .iter()
            .map(|credential| {
                let (partial_mdoc, identifier) = credential.to_partial_mdoc(issuer_keypair, wscd);

                (credential.query_id.clone(), vec_nonempty![(partial_mdoc, identifier)])
            })
            .collect()
    }

    pub fn to_unsigned_sd_jwt_presentations(
        &self,
        issuer_keypair: &KeyPair,
        wscd: &impl AsRef<MockRemoteWscd>,
    ) -> HashMap<CredentialQueryIdentifier, VecNonEmpty<(UnsignedSdJwtPresentation, String)>> {
        self.as_ref()
            .iter()
            .map(|credential| {
                let (presentation, identifier) = credential.to_unsigned_sd_jwt_presentation(issuer_keypair, wscd);

                (credential.query_id.clone(), vec_nonempty![(presentation, identifier)])
            })
            .collect()
    }

    pub fn assert_matches_disclosed_attestations(
        &self,
        disclosed_attestations: &UniqueIdVec<DisclosedAttestations>,
        expected_formats: impl IntoIterator<Item = Format>,
    ) {
        for ((credential, disclosed), expected_format) in self
            .as_ref()
            .iter()
            .zip_eq(disclosed_attestations.as_ref())
            .zip_eq(expected_formats)
        {
            // The credential query identifier should match, in the same order as the request.
            assert_eq!(disclosed.id, credential.query_id);

            // The response should contain exactly one attestation.
            assert_eq!(disclosed.attestations.len().get(), 1);

            let attestation = disclosed.attestations.first();

            // Verify the attestation type.
            assert_eq!(
                attestation.attestation_type,
                credential.sd_jwt_payload_preview.attestation_type
            );

            // Verify the actual attributes.
            credential.assert_matches_disclosed_attributes(&attestation.attributes, expected_format);
        }
    }
}

impl AsRef<[TestCredential]> for TestCredentials {
    fn as_ref(&self) -> &[TestCredential] {
        let Self(credentials) = self;

        credentials.as_ref()
    }
}

impl Add for TestCredentials {
    type Output = TestCredentials;

    fn add(self, rhs: Self) -> Self::Output {
        let mut credentials = self.0;
        credentials.extend(rhs.0.into_inner());
        Self::new(credentials)
    }
}

impl TestCredential {
    #[expect(clippy::too_many_arguments, reason = "test constructor")]
    pub fn new<'a>(
        sd_jwt_payload_preview: PreviewableCredentialPayload,
        mdoc_attributes: Attributes,
        type_metadata_documents: Option<TypeMetadataDocuments>,
        query_id: CredentialQueryIdentifier,
        query_claim_paths: impl IntoIterator<Item = impl IntoIterator<Item = &'a str>>,
        mdoc_query_claim_paths: impl IntoIterator<Item = impl IntoIterator<Item = &'a str>>,
        status: StatusClaim,
    ) -> Self {
        fn to_claim_paths<'a>(
            paths: impl IntoIterator<Item = impl IntoIterator<Item = &'a str>>,
        ) -> Vec<VecNonEmpty<ClaimPath>> {
            paths
                .into_iter()
                .map(|path| {
                    path.into_iter()
                        .map(|element| ClaimPath::SelectByKey(element.to_string()))
                        .collect_vec()
                        .try_into()
                        .expect("query path should have at least one element")
                })
                .collect_vec()
        }

        let mut sd_jwt_disclosure_attributes = sd_jwt_payload_preview.attributes.clone();
        sd_jwt_disclosure_attributes.prune(&to_claim_paths(query_claim_paths));

        let mut mdoc_disclosure_attributes = mdoc_attributes.clone();
        mdoc_disclosure_attributes.prune(&to_claim_paths(mdoc_query_claim_paths));

        Self {
            sd_jwt_payload_preview,
            mdoc_attributes,
            type_metadata_documents,
            query_id,
            sd_jwt_disclosure_attributes,
            mdoc_disclosure_attributes,
            status,
        }
    }

    pub fn to_normalized_credential_request(&self, format: Format) -> NormalizedCredentialRequest {
        match format {
            Format::MsoMdoc => self.to_mdoc_normalized_credential_request(),
            Format::SdJwt => self.to_sd_jwt_normalized_credential_request(),
        }
    }

    fn to_mdoc_attributes(&self) -> IndexMap<String, Vec<Entry>> {
        self.mdoc_disclosure_attributes
            .clone()
            .to_mdoc_attributes()
            .expect("mdoc attributes of a TestCredential should be laid out in namespaces")
    }

    fn to_mdoc_claim_paths(&self) -> impl Iterator<Item = VecNonEmpty<ClaimPath>> {
        self.to_mdoc_attributes().into_iter().flat_map(|(name_space, entries)| {
            std::iter::repeat_n(name_space, entries.len())
                .zip(entries)
                .map(|(name_space, entry)| {
                    vec_nonempty![
                        ClaimPath::SelectByKey(name_space.clone()),
                        ClaimPath::SelectByKey(entry.name),
                    ]
                })
        })
    }

    fn to_mdoc_normalized_credential_request(&self) -> NormalizedCredentialRequest {
        let claims = self
            .to_mdoc_claim_paths()
            .map(|path| MdocAttributeRequest {
                path,
                intent_to_retain: None,
            })
            .collect_vec()
            .try_into()
            .expect("TestCredential payload preview should have at least one attribute");

        NormalizedCredentialRequest::MsoMdoc {
            id: self.query_id.clone(),
            doctype_value: self.sd_jwt_payload_preview.attestation_type.clone(),
            claims,
            aki: vec![],
        }
    }

    fn to_sd_jwt_normalized_credential_request(&self) -> NormalizedCredentialRequest {
        let claims = self
            .sd_jwt_disclosure_attributes
            .claim_paths(AttributesTraversalBehaviour::OnlyLeaves)
            .into_iter()
            .map(|path| SdJwtAttributeRequest { path })
            .collect_vec()
            .try_into()
            .expect("TestCredential payload preview should have at least one attribute");

        NormalizedCredentialRequest::SdJwt {
            id: self.query_id.clone(),
            vct_values: vec_nonempty![self.sd_jwt_payload_preview.attestation_type.clone()],
            claims,
            aki: vec![],
        }
    }

    fn to_credential_payload(
        &self,
        wscd: &impl AsRef<MockRemoteWscd>,
        attributes: Attributes,
        vct_integrity: Option<Integrity>,
    ) -> (CredentialPayload, String) {
        let holder_key = wscd.as_ref().create_random_key();

        let credential_payload = CredentialPayload::from_previewable_credential_payload(
            PreviewableCredentialPayload {
                attributes,
                ..self.sd_jwt_payload_preview.clone()
            },
            Utc::now(),
            &PublicKey::from(*holder_key.verifying_key()),
            vct_integrity,
            self.status.clone(),
        )
        .expect("TestCredential payload preview should convert to CredentialPayload");

        (credential_payload, holder_key.identifier)
    }

    pub fn to_mdoc(&self, issuer_keypair: &KeyPair, wscd: &impl AsRef<MockRemoteWscd>) -> (Mdoc, String) {
        let (credential_payload, holder_key_identifier) =
            self.to_credential_payload(wscd, self.mdoc_attributes.clone(), None);

        let (issuer_signed, mso) = credential_payload
            .into_signed_mdoc(issuer_keypair)
            .now_or_never()
            .unwrap()
            .expect("TestCredential payload preview should convert to Mdoc");

        (Mdoc::new_unverified(mso, issuer_signed), holder_key_identifier)
    }

    pub fn to_sd_jwt(&self, issuer_keypair: &KeyPair, wscd: &impl AsRef<MockRemoteWscd>) -> (SignedSdJwt, String) {
        let type_metadata_documents = self
            .type_metadata_documents
            .clone()
            .expect("TestCredential should carry Type Metadata documents to be issued as an SD-JWT");

        let vct_integrity = Integrity::from(type_metadata_documents.as_ref().first());
        let (normalized_metadata, _) = type_metadata_documents
            .into_normalized(&self.sd_jwt_payload_preview.attestation_type)
            .expect("TestCredential Type Metadata documents should normalize");

        let (credential_payload, holder_key_identifier) = self.to_credential_payload(
            wscd,
            self.sd_jwt_payload_preview.attributes.clone(),
            Some(vct_integrity),
        );

        let sd_jwt = credential_payload
            .into_signed_sd_jwt(&normalized_metadata, issuer_keypair)
            .now_or_never()
            .unwrap()
            .expect("TestCredential payload preview should convert to SD-JWT");

        (sd_jwt, holder_key_identifier)
    }

    pub fn to_partial_mdoc(
        &self,
        issuer_keypair: &KeyPair,
        wscd: &impl AsRef<MockRemoteWscd>,
    ) -> (PartialMdoc, String) {
        let (mdoc, holder_key_identifier) = self.to_mdoc(issuer_keypair, wscd);
        let claim_paths = self.to_mdoc_claim_paths().collect_vec();

        let partial_mdoc = PartialMdoc::try_new(mdoc, &claim_paths)
            .expect("TestCredential payload preview should convert to PartialMdoc");

        (partial_mdoc, holder_key_identifier)
    }

    pub fn to_unsigned_sd_jwt_presentation(
        &self,
        issuer_keypair: &KeyPair,
        wscd: &impl AsRef<MockRemoteWscd>,
    ) -> (UnsignedSdJwtPresentation, String) {
        let (signed_sd_jwt, identifier) = self.to_sd_jwt(issuer_keypair, wscd);

        let sd_jwt = signed_sd_jwt.into_verified();
        let presentation = self
            .sd_jwt_disclosure_attributes
            .claim_paths(AttributesTraversalBehaviour::OnlyLeaves)
            .iter()
            .fold(sd_jwt.into_presentation_builder(), |builder, path| {
                builder.disclose(path).unwrap()
            })
            .finish();

        (presentation, identifier)
    }

    pub fn assert_matches_disclosed_attributes(
        &self,
        disclosed_attributes: &DisclosedAttributes,
        expected_format: Format,
    ) {
        match (&disclosed_attributes, expected_format) {
            (DisclosedAttributes::MsoMdoc(attributes), Format::MsoMdoc) => {
                let expected_attributes = self
                    .to_mdoc_attributes()
                    .into_iter()
                    .map(|(name_space, entries)| {
                        let name_space_attributes = entries
                            .into_iter()
                            .map(|entry| (entry.name, Attribute::try_from(entry.value).unwrap()))
                            .collect::<IndexMap<_, _>>();

                        (name_space, name_space_attributes)
                    })
                    .collect::<IndexMap<_, _>>();

                assert_eq!(*attributes, expected_attributes);
            }
            (DisclosedAttributes::SdJwt(attributes), Format::SdJwt) => {
                let disclosed_paths = attributes
                    .claim_paths(AttributesTraversalBehaviour::OnlyLeaves)
                    .into_iter()
                    .collect::<HashSet<_>>();
                let expected_paths = self
                    .sd_jwt_disclosure_attributes
                    .claim_paths(AttributesTraversalBehaviour::OnlyLeaves)
                    .into_iter()
                    .collect::<HashSet<_>>();

                assert_eq!(disclosed_paths, expected_paths);

                for path in &disclosed_paths {
                    let disclosed_attribute = attributes.get(path).unwrap().unwrap();
                    let expected_attribute = self.sd_jwt_disclosure_attributes.get(path).unwrap().unwrap();

                    assert_eq!(disclosed_attribute, expected_attribute);
                }
            }
            _ => panic!("disclosed attestation is not in expected format"),
        }
    }
}

impl TestCredential {
    fn new_nl_pid<'a>(
        query_id: &str,
        query_claim_paths: impl IntoIterator<Item = impl IntoIterator<Item = &'a str>>,
        mdoc_query_claim_paths: impl IntoIterator<Item = impl IntoIterator<Item = &'a str>>,
    ) -> Self {
        let (_, type_metadata_documents) = TypeMetadataDocuments::nl_pid_example();

        Self::new(
            PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            Attributes::nl_pid_mdoc_example(),
            Some(type_metadata_documents),
            query_id.parse().unwrap(),
            query_claim_paths,
            mdoc_query_claim_paths,
            StatusClaim::new_mock(),
        )
    }

    pub fn new_nl_pid_all() -> Self {
        Self::new_nl_pid(
            "nl_pid_all",
            [[PID_GIVEN_NAME], [PID_FAMILY_NAME], [PID_BIRTH_DATE], [PID_BSN]],
            [
                [PID_ATTESTATION_TYPE, PID_GIVEN_NAME],
                [PID_ATTESTATION_TYPE, PID_FAMILY_NAME],
                [PID_ATTESTATION_TYPE, PID_BIRTH_DATE],
                [PID_ATTESTATION_TYPE, PID_BSN],
            ],
        )
    }

    pub fn new_nl_pid_full_name() -> Self {
        Self::new_nl_pid(
            "nl_pid_full_name",
            [[PID_GIVEN_NAME], [PID_FAMILY_NAME]],
            [
                [PID_ATTESTATION_TYPE, PID_GIVEN_NAME],
                [PID_ATTESTATION_TYPE, PID_FAMILY_NAME],
            ],
        )
    }

    pub fn new_nl_pid_given_name() -> Self {
        Self::new_nl_pid(
            "nl_pid_given_name",
            [[PID_GIVEN_NAME]],
            [[PID_ATTESTATION_TYPE, PID_GIVEN_NAME]],
        )
    }

    pub fn new_nl_pid_given_name_for_query_id(query_id: &str) -> Self {
        Self::new_nl_pid(query_id, [[PID_GIVEN_NAME]], [[PID_ATTESTATION_TYPE, PID_GIVEN_NAME]])
    }

    pub fn new_nl_pid_family_name() -> Self {
        Self::new_nl_pid(
            "nl_pid_family_name",
            [[PID_FAMILY_NAME]],
            [[PID_ATTESTATION_TYPE, PID_FAMILY_NAME]],
        )
    }

    pub fn new_nl_pid_address<'a>(
        query_id: &str,
        query_claim_paths: impl IntoIterator<Item = impl IntoIterator<Item = &'a str>>,
        mdoc_query_claim_paths: impl IntoIterator<Item = impl IntoIterator<Item = &'a str>>,
    ) -> Self {
        let (_, type_metadata_documents) = TypeMetadataDocuments::nl_address_example();

        Self::new(
            PreviewableCredentialPayload::nl_pid_address_example(&MockTimeGenerator::default()),
            Attributes::nl_pid_address_mdoc_example(),
            Some(type_metadata_documents),
            query_id.parse().unwrap(),
            query_claim_paths,
            mdoc_query_claim_paths,
            StatusClaim::new_mock(),
        )
    }

    pub fn new_nl_pid_address_all() -> Self {
        Self::new_nl_pid_address(
            "nl_pid_address_all",
            [
                [PID_ADDRESS_GROUP, PID_RESIDENT_STREET],
                [PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER],
                [PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE],
                [PID_ADDRESS_GROUP, PID_RESIDENT_CITY],
                [PID_ADDRESS_GROUP, PID_RESIDENT_COUNTRY],
            ],
            [
                [ADDRESS_ATTESTATION_TYPE, PID_RESIDENT_STREET],
                [ADDRESS_ATTESTATION_TYPE, PID_RESIDENT_HOUSE_NUMBER],
                [ADDRESS_ATTESTATION_TYPE, PID_RESIDENT_POSTAL_CODE],
                [ADDRESS_ATTESTATION_TYPE, PID_RESIDENT_CITY],
                [ADDRESS_ATTESTATION_TYPE, PID_RESIDENT_COUNTRY],
            ],
        )
    }

    pub fn new_nl_pid_address_minimal_address() -> Self {
        Self::new_nl_pid_address(
            "nl_pid_address_minimal_address",
            [
                [PID_ADDRESS_GROUP, PID_RESIDENT_STREET],
                [PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER],
                [PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE],
            ],
            [
                [ADDRESS_ATTESTATION_TYPE, PID_RESIDENT_STREET],
                [ADDRESS_ATTESTATION_TYPE, PID_RESIDENT_HOUSE_NUMBER],
                [ADDRESS_ATTESTATION_TYPE, PID_RESIDENT_POSTAL_CODE],
            ],
        )
    }
}

pub fn nl_pid_credentials_all() -> TestCredentials {
    TestCredentials::new(vec_nonempty![TestCredential::new_nl_pid_all()])
}

pub fn nl_pid_credentials_full_name() -> TestCredentials {
    TestCredentials::new(vec_nonempty![TestCredential::new_nl_pid_full_name()])
}

pub fn nl_pid_credentials_given_name() -> TestCredentials {
    TestCredentials::new(vec_nonempty![TestCredential::new_nl_pid_given_name()])
}

pub fn nl_pid_credentials_given_name_for_query_id(query_id: &str) -> TestCredentials {
    TestCredentials::new(vec_nonempty![TestCredential::new_nl_pid_given_name_for_query_id(
        query_id
    )])
}

pub fn nl_pid_credentials_family_name() -> TestCredentials {
    TestCredentials::new(vec_nonempty![TestCredential::new_nl_pid_family_name()])
}

pub fn nl_pid_address_credentials_all() -> TestCredentials {
    TestCredentials::new(vec_nonempty![TestCredential::new_nl_pid_address_all()])
}

pub fn nl_pid_address_minimal_address() -> TestCredentials {
    TestCredentials::new(vec_nonempty![TestCredential::new_nl_pid_address_minimal_address()])
}
