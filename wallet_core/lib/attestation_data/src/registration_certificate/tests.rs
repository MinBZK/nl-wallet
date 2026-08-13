use std::assert_matches;
use std::sync::Arc;
use std::time::Duration;

use attestation_types::status_claim::StatusListClaim;
use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use crypto::server_keys::generate::Ca;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::CanonicalDistinguishedName;
use crypto::x509::DistinguishedName;
use dcql::CredentialQueryFormat;
use jwt::error::JwtParseError;
use rstest::rstest;
use serde_json::Value;
use serde_json::json;
use token_status_list::status_list::StatusList;
use token_status_list::status_list::StatusType;
use token_status_list::status_list_token::StatusListToken;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::client::StatusListClientError;
use token_status_list::verification::verifier::RevocationVerifier;
use url::Url;
use utils::generator::mock::MockTimeGenerator;
use utils::vec_nonempty;

use super::*;
use crate::x509::RelyingParty;

const ANNEX_C_EXAMPLE: &str = include_str!("../../examples/spec/registration_certificate_annex_c.json");
const STATUS_LIST_URI: &str = "https://example.com/statuslists/1";

#[derive(Debug)]
struct StaticStatusListClient(StatusListToken);

impl StatusListClient for StaticStatusListClient {
    async fn fetch(&self, _url: Url) -> Result<StatusListToken, StatusListClientError> {
        Ok(self.0.clone())
    }
}

#[derive(Debug)]
struct FailingStatusListClient;

impl StatusListClient for FailingStatusListClient {
    async fn fetch(&self, _url: Url) -> Result<StatusListToken, StatusListClientError> {
        Err(StatusListClientError::JwtParsing(JwtParseError::MissingKid))
    }
}

struct StatusValidationContext {
    verifier: RevocationVerifier<StaticStatusListClient>,
    trust_anchors: TrustAnchors,
    signing_certificate_dn: CanonicalDistinguishedName,
    time: MockTimeGenerator,
}

async fn status_validation_context(status: StatusType, status_list_length: usize) -> StatusValidationContext {
    let ca = Ca::generate_mock();
    let status_list_signer = ca.generate_issuer_status_list_mock().unwrap();
    let registration_certificate_signer = ca.generate_issuer_mock().unwrap();
    let mut status_list = StatusList::new(status_list_length);
    if status != StatusType::Valid {
        assert_eq!(status_list.insert(0, status), None);
    }
    let status_list_token = StatusListToken::builder(STATUS_LIST_URI.parse().unwrap(), status_list.pack())
        .sign(&status_list_signer)
        .await
        .unwrap();
    let time = MockTimeGenerator::default();

    StatusValidationContext {
        verifier: RevocationVerifier::new(
            Arc::new(StaticStatusListClient(status_list_token)),
            0,
            Duration::ZERO,
            Duration::ZERO,
            time.clone(),
        ),
        trust_anchors: TrustAnchors::from(&ca),
        signing_certificate_dn: registration_certificate_signer
            .certificate()
            .to_canonical_distinguished_name()
            .unwrap(),
        time,
    }
}

fn valid_payload_json() -> Value {
    let mut payload: Value = serde_json::from_str(ANNEX_C_EXAMPLE).unwrap();
    payload["id"] = json!("wrprc-example-1");
    payload["status"] = json!({
        "idx": "0",
        "uri": STATUS_LIST_URI,
    });
    payload
}

fn legal_person_access_certificate_subject() -> RelyingParty {
    RelyingParty::try_from(DistinguishedName::new_legal_person(
        "Example Company".to_string(),
        "DE".to_string(),
        "Example Company GmbH".to_string(),
        "LEIXG-529900T8BM49AURSDO55".to_string(),
    ))
    .unwrap()
}

fn valid_payload() -> UncheckedRegistrationCertificate {
    serde_json::from_value(valid_payload_json()).unwrap()
}

fn validation_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2023, 5, 3, 0, 0, 0).unwrap()
}

#[test]
fn parse_annex_c_example_and_reject_its_missing_normative_id() {
    let json: Value = serde_json::from_str(ANNEX_C_EXAMPLE).unwrap();
    assert_eq!(
        json["credentials"][0]["claim"][0]["path"],
        json!(["age_equal_or_over", "18"])
    );
    let payload: UncheckedRegistrationCertificate = serde_json::from_str(ANNEX_C_EXAMPLE).unwrap();

    assert_eq!(payload.srv_description.0.as_slice().len(), 1);
    assert_matches!(
        &payload.credentials.as_ref().unwrap()[0].format,
        CredentialQueryFormat::SdJwt { .. }
    );
    assert_eq!(
        payload.intermediary.as_ref().unwrap().sname.as_deref(),
        Some("Intermediary Services Ltd.")
    );
    assert_matches!(
        payload
            .clone()
            .validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::MissingId)
    );

    let mut payload_with_id = payload;
    payload_with_id.id = Some("wrprc-example-1".to_string());
    assert_matches!(
        payload_with_id.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::InvalidStatusWireFormat)
    );
}

#[test]
fn validate_payload_based_on_annex_c_example() {
    let certificate = valid_payload()
        .validate_structure(&legal_person_access_certificate_subject(), validation_time())
        .unwrap();

    assert_eq!(certificate.id(), "wrprc-example-1");
    assert_eq!(certificate.subject_type(), SubjectType::LegalPerson);
}

#[test]
fn accept_flat_service_description_from_ticket_validation_shape() {
    let mut json = valid_payload_json();
    json["srv_description"] = json!([{"lang": "en-US", "value": "Service"}]);

    let payload: UncheckedRegistrationCertificate = serde_json::from_value(json).unwrap();
    assert_eq!(payload.srv_description.0.as_slice().len(), 1);
    assert_eq!(payload.srv_description.0[0].as_slice().len(), 1);
}

#[test]
fn validate_natural_person_subject_binding() {
    let mut json = valid_payload_json();
    let object = json.as_object_mut().unwrap();
    object.remove("sub_ln");
    object.insert("sub".to_string(), json!("12345678"));
    object.insert("sub_gn".to_string(), json!("Jane"));
    object.insert("sub_fn".to_string(), json!("Doe"));
    let payload: UncheckedRegistrationCertificate = serde_json::from_value(json).unwrap();
    let access_subject = RelyingParty::try_from(DistinguishedName::new_natural_person(
        "Jane Doe".to_string(),
        "DE".to_string(),
        "12345678".to_string(),
        "Doe".to_string(),
        "Jane".to_string(),
    ))
    .unwrap();

    let certificate = payload.validate_structure(&access_subject, validation_time()).unwrap();
    assert_eq!(certificate.subject_type(), SubjectType::NaturalPerson);
}

#[test]
fn reject_ambiguous_subject_type() {
    let mut payload = valid_payload();
    payload.sub_gn = Some("Jane".to_string());
    payload.sub_fn = Some("Doe".to_string());

    assert_matches!(
        payload.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::AmbiguousSubjectType)
    );
}

#[test]
fn reject_subject_identifier_mismatch() {
    let mut payload = valid_payload();
    payload.sub = "different".to_string();

    assert_matches!(
        payload.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::SubjectIdentifierMismatch {
            field: "organizationIdentifier"
        })
    );
}

#[test]
fn reject_access_certificate_subject_type_mismatch() {
    let payload = valid_payload();
    let access_subject = RelyingParty::try_from(DistinguishedName::new_natural_person(
        "Jane Doe".to_string(),
        "DE".to_string(),
        "12345678".to_string(),
        "Doe".to_string(),
        "Jane".to_string(),
    ))
    .unwrap();

    assert_matches!(
        payload.validate_structure(&access_subject, validation_time()),
        Err(
            RegistrationCertificateValidationError::AccessCertificateSubjectTypeMismatch {
                registration: SubjectType::LegalPerson,
                access: SubjectType::NaturalPerson,
            }
        )
    );
}

#[test]
fn service_provider_requires_credentials_and_purpose() {
    let mut payload = valid_payload();
    payload.entitlements = vec_nonempty![SERVICE_PROVIDER_ENTITLEMENT.parse().unwrap()];
    payload.credentials = None;

    assert_matches!(
        payload
            .clone()
            .validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::MissingServiceProviderCredentials)
    );

    payload.credentials = Some(Vec::new());
    payload.purpose = None;
    assert_matches!(
        payload.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::MissingServiceProviderPurpose)
    );
}

#[test]
fn reject_unrecognized_entitlement() {
    let mut payload = valid_payload();
    payload.entitlements = vec_nonempty!["https://example.com/unknown-entitlement".parse().unwrap()];

    assert_matches!(
        payload.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::InvalidEntitlement { .. })
    );
}

#[test]
fn reject_sub_entitlement_without_annex_a_2_entitlement() {
    let mut payload = valid_payload();
    payload.entitlements = vec_nonempty!["https://uri.etsi.org/19475/SubEntitlement/psp/psp-ai".parse().unwrap(),];

    assert_matches!(
        payload.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::MissingAnnexA2Entitlement)
    );
}

#[test]
fn accept_normative_annex_a_3_1_sub_entitlements() {
    for sub_entitlement in ANNEX_A_3_1_SUB_ENTITLEMENTS {
        let mut payload = valid_payload();
        payload.entitlements = vec_nonempty![
            "https://uri.etsi.org/19475/Entitlement/Non_Q_EAA_Provider"
                .parse()
                .unwrap(),
            sub_entitlement.parse().unwrap(),
        ];

        payload
            .validate_structure(&legal_person_access_certificate_subject(), validation_time())
            .unwrap();
    }
}

#[test]
fn reject_unknown_sub_entitlement() {
    let mut payload = valid_payload();
    payload.entitlements = vec_nonempty![
        "https://uri.etsi.org/19475/Entitlement/Non_Q_EAA_Provider"
            .parse()
            .unwrap(),
        "https://uri.etsi.org/19475/SubEntitlement/psp/not-defined"
            .parse()
            .unwrap(),
    ];

    assert_matches!(
        payload.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::InvalidEntitlement { index: 1, .. })
    );
}

#[test]
fn validate_expiration_window() {
    let mut payload = valid_payload();
    payload.iat = Utc.with_ymd_and_hms(2024, 2, 29, 12, 0, 0).unwrap();
    payload.exp = Some(Utc.with_ymd_and_hms(2025, 2, 28, 12, 0, 0).unwrap());
    let now = Utc.with_ymd_and_hms(2025, 2, 28, 11, 59, 59).unwrap();
    payload
        .clone()
        .validate_structure(&legal_person_access_certificate_subject(), now)
        .unwrap();

    payload.exp = Some(Utc.with_ymd_and_hms(2025, 2, 28, 12, 0, 1).unwrap());
    assert_matches!(
        payload
            .clone()
            .validate_structure(&legal_person_access_certificate_subject(), now),
        Err(RegistrationCertificateValidationError::ExpirationTooLate)
    );

    payload.exp = Some(now);
    assert_matches!(
        payload.validate_structure(&legal_person_access_certificate_subject(), now),
        Err(RegistrationCertificateValidationError::Expired { .. })
    );
}

#[test]
fn parse_compatible_status_shapes_and_serialize_canonical_shape() {
    let direct: RegistrationCertificateStatus = serde_json::from_value(json!({
        "idx": "42",
        "uri": "https://example.com/statuslists/1"
    }))
    .unwrap();
    let oauth: RegistrationCertificateStatus = serde_json::from_value(json!({
        "status_list": {
            "idx": 42,
            "uri": "https://example.com/statuslists/1"
        }
    }))
    .unwrap();

    assert_eq!(direct.status_list_claim(), oauth.status_list_claim());
    assert_eq!(direct.status_list_claim().idx, 42);
    let canonical = json!({
        "idx": "42",
        "uri": "https://example.com/statuslists/1"
    });
    assert_eq!(serde_json::to_value(direct).unwrap(), canonical);
    assert_eq!(serde_json::to_value(oauth).unwrap(), canonical);
}

#[test]
fn reject_non_numeric_status_list_index() {
    for index in ["", "+42", "-42", "42.0", "forty-two"] {
        let result = serde_json::from_value::<RegistrationCertificateStatus>(json!({
            "idx": index,
            "uri": STATUS_LIST_URI,
        }));
        assert!(result.is_err(), "`{index}` should not be a numeric string");
    }
}

#[test]
fn reject_non_normative_status_shapes_during_structural_validation() {
    let mut nested = valid_payload_json();
    nested["status"] = json!({
        "status_list": {
            "idx": 0,
            "uri": STATUS_LIST_URI,
        }
    });
    let nested: UncheckedRegistrationCertificate = serde_json::from_value(nested).unwrap();
    assert_matches!(
        nested.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::InvalidStatusWireFormat)
    );

    let mut integer = valid_payload_json();
    integer["status"] = json!({
        "idx": 0,
        "uri": STATUS_LIST_URI,
    });
    let integer: UncheckedRegistrationCertificate = serde_json::from_value(integer).unwrap();
    assert_matches!(
        integer.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::InvalidStatusWireFormat)
    );
}

#[test]
fn validate_multi_language_strings() {
    for language_tag in [
        "en",
        "en-US",
        "zh-Hant-TW",
        "de-CH-1901",
        "sl-rozaj-biske",
        "en-a-aaa-b-ccc-x-private",
        "i-klingon",
        "x-private",
    ] {
        let value = vec_nonempty![MultiLanguageString {
            lang: language_tag.to_string(),
            value: "value".to_string(),
        }];
        validate_multi_language_string_set(&value).unwrap();
    }

    for language_tag in ["", "e", "en_US", "en-", "en-US-abc", "en-a", "en-x", "en-ü"] {
        let value = vec_nonempty![MultiLanguageString {
            lang: language_tag.to_string(),
            value: "value".to_string(),
        }];
        assert_matches!(
            validate_multi_language_string_set(&value),
            Err(MultiLanguageStringSetValidationError::InvalidLanguageTag { .. }),
            "{language_tag}"
        );
    }

    let empty_value = vec_nonempty![MultiLanguageString {
        lang: "en".to_string(),
        value: "  ".to_string(),
    }];
    assert_matches!(
        validate_multi_language_string_set(&empty_value),
        Err(MultiLanguageStringSetValidationError::EmptyValue { index: 0 })
    );
}

#[test]
fn validate_credential_metadata() {
    validate_credential_set(&[]).unwrap();

    let empty_doctype: Credential = serde_json::from_value(json!({
        "format": "mso_mdoc",
        "meta": { "doctype_value": " " }
    }))
    .unwrap();
    assert_matches!(
        validate_credential_set(&[empty_doctype]),
        Err(CredentialSetValidationError::EmptyDoctype { index: 0 })
    );

    let empty_vct: Credential = serde_json::from_value(json!({
        "format": "dc+sd-jwt",
        "meta": { "vct_values": ["urn:eudi:pid:1", " "] }
    }))
    .unwrap();
    assert_matches!(
        validate_credential_set(&[empty_vct]),
        Err(CredentialSetValidationError::EmptyVctValue {
            index: 0,
            value_index: 1,
        })
    );
}

#[test]
fn credential_claim_values_reject_other_json_types() {
    let valid: Credential = serde_json::from_value(json!({
        "format": "dc+sd-jwt",
        "meta": { "vct_values": ["urn:eudi:pid:1"] },
        "claim": [{ "path": ["age"], "values": ["18", 18, true] }]
    }))
    .unwrap();
    assert_eq!(valid.claim.unwrap()[0].values.len(), 3);

    for invalid_value in [json!(null), json!(1.5), json!([]), json!({})] {
        let result = serde_json::from_value::<Credential>(json!({
            "format": "dc+sd-jwt",
            "meta": { "vct_values": ["urn:eudi:pid:1"] },
            "claim": [{ "path": ["age"], "values": [invalid_value] }]
        }));
        assert!(result.is_err(), "value should be rejected");
    }
}

#[test]
fn reject_invalid_contact_fields() {
    let mut invalid_support = valid_payload();
    invalid_support.support_uri = "not a URL or email address".to_string();
    assert_matches!(
        invalid_support.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::InvalidSupportUri)
    );

    let mut invalid_email = valid_payload();
    invalid_email.supervisory_authority.email = Some("supervisory.example.com".to_string());
    assert_matches!(
        invalid_email.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::InvalidSupervisoryAuthorityEmail)
    );

    for phone in ["49 123 4567890", "+49 CALL-ME"] {
        let mut invalid_phone = valid_payload();
        invalid_phone.supervisory_authority.phone = Some(phone.to_string());
        assert_matches!(
            invalid_phone.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
            Err(RegistrationCertificateValidationError::InvalidSupervisoryAuthorityPhone)
        );
    }
}

#[rstest]
#[case("registry_uri")]
#[case("info_uri")]
#[case("privacy_policy")]
#[case("certificate_policy")]
fn reject_malformed_top_level_url(#[case] field: &str) {
    let mut json = valid_payload_json();
    json[field] = json!("not a URL");

    assert!(serde_json::from_value::<UncheckedRegistrationCertificate>(json).is_err());
}

#[test]
fn reject_malformed_nested_urls() {
    let mut invalid_authority = valid_payload_json();
    invalid_authority["supervisory_authority"]["uri"] = json!("not a URL");
    assert!(serde_json::from_value::<UncheckedRegistrationCertificate>(invalid_authority).is_err());

    let mut invalid_status = valid_payload_json();
    invalid_status["status"]["uri"] = json!("not a URL");
    assert!(serde_json::from_value::<UncheckedRegistrationCertificate>(invalid_status).is_err());

    let mut invalid_annex_c_status: Value = serde_json::from_str(ANNEX_C_EXAMPLE).unwrap();
    invalid_annex_c_status["status"]["status_list"]["uri"] = json!("not a URL");
    assert!(serde_json::from_value::<UncheckedRegistrationCertificate>(invalid_annex_c_status).is_err());
}

#[test]
fn accept_url_schemes_independent_of_application_http_policy() {
    let mut payload = valid_payload();
    payload.registry_uri = "ftp://registry.example.com".parse().unwrap();
    payload.privacy_policy = Some("ftp://example.com/privacy".parse().unwrap());
    payload.certificate_policy = "ftp://registrar.example.com/certificate-policy".parse().unwrap();

    payload
        .validate_structure(&legal_person_access_certificate_subject(), validation_time())
        .unwrap();
}

#[test]
fn reject_invalid_core_values() {
    let mut invalid_id = valid_payload();
    invalid_id.id = Some("  ".to_string());
    assert_matches!(
        invalid_id.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::EmptyField { field: "id" })
    );

    let mut invalid_name = valid_payload();
    invalid_name.name = Some("  ".to_string());
    assert_matches!(
        invalid_name.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::EmptyField { field: "name" })
    );

    let mut invalid_country = valid_payload();
    invalid_country.country = "NLD".to_string();
    assert_matches!(
        invalid_country.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::InvalidCountry)
    );

    let mut invalid_policy = valid_payload();
    invalid_policy.policy_id.clear();
    assert_matches!(
        invalid_policy.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
        Err(RegistrationCertificateValidationError::MissingPolicyIdentifier)
    );
}

#[test]
fn status_converts_to_existing_status_list_claim() {
    let expected = StatusListClaim {
        idx: 1,
        uri: "https://example.com/statuslists/1".parse().unwrap(),
    };
    let status = RegistrationCertificateStatus::from(expected.clone());
    assert_eq!(StatusListClaim::from(status), expected);
}

#[tokio::test]
async fn accept_registration_certificate_with_valid_referenced_status() {
    let context = status_validation_context(StatusType::Valid, 8).await;
    let certificate = valid_payload()
        .validate_structure(&legal_person_access_certificate_subject(), validation_time())
        .unwrap()
        .validate_status(
            &context.verifier,
            &context.trust_anchors,
            context.signing_certificate_dn,
            &context.time,
        )
        .await
        .unwrap();

    assert_eq!(certificate.id(), "wrprc-example-1");
}

#[tokio::test]
async fn reject_registration_certificate_with_revoked_status() {
    let context = status_validation_context(StatusType::Invalid, 8).await;
    let result = valid_payload()
        .validate_structure(&legal_person_access_certificate_subject(), validation_time())
        .unwrap()
        .validate_status(
            &context.verifier,
            &context.trust_anchors,
            context.signing_certificate_dn,
            &context.time,
        )
        .await;

    assert_matches!(result, Err(RegistrationCertificateStatusValidationError::NotValid));
}

#[tokio::test]
async fn reject_registration_certificate_with_out_of_bounds_status_index() {
    let context = status_validation_context(StatusType::Valid, 8).await;
    let mut payload = valid_payload();
    payload.status = StatusListClaim {
        idx: 8,
        uri: STATUS_LIST_URI.parse().unwrap(),
    }
    .into();
    let result = payload
        .validate_structure(&legal_person_access_certificate_subject(), validation_time())
        .unwrap()
        .validate_status(
            &context.verifier,
            &context.trust_anchors,
            context.signing_certificate_dn,
            &context.time,
        )
        .await;

    assert_matches!(
        result,
        Err(RegistrationCertificateStatusValidationError::InvalidStatusListReference)
    );
}

#[tokio::test]
async fn reject_status_list_not_signed_under_registration_certificate_trust_anchor() {
    let context = status_validation_context(StatusType::Valid, 8).await;
    let untrusted_ca = Ca::generate_mock();
    let result = valid_payload()
        .validate_structure(&legal_person_access_certificate_subject(), validation_time())
        .unwrap()
        .validate_status(
            &context.verifier,
            &TrustAnchors::from(&untrusted_ca),
            context.signing_certificate_dn,
            &context.time,
        )
        .await;

    assert_matches!(
        result,
        Err(RegistrationCertificateStatusValidationError::InvalidStatusListReference)
    );
}

#[tokio::test]
async fn reject_registration_certificate_when_status_cannot_be_determined() {
    let context = status_validation_context(StatusType::Valid, 8).await;
    let verifier = RevocationVerifier::new(
        Arc::new(FailingStatusListClient),
        0,
        Duration::ZERO,
        Duration::ZERO,
        context.time.clone(),
    );
    let result = valid_payload()
        .validate_structure(&legal_person_access_certificate_subject(), validation_time())
        .unwrap()
        .validate_status(
            &verifier,
            &context.trust_anchors,
            context.signing_certificate_dn,
            &context.time,
        )
        .await;

    assert_matches!(result, Err(RegistrationCertificateStatusValidationError::Undetermined));
}

#[test]
fn registration_certificate_payload_types_use_jades_type() {
    assert_eq!(
        <UncheckedRegistrationCertificate as jwt::JwtTyp>::TYP,
        jwt::jades_b_b::JADES_B_B_JWT_TYP
    );
    assert_eq!(
        <StructurallyValidatedRegistrationCertificate as jwt::JwtTyp>::TYP,
        jwt::jades_b_b::JADES_B_B_JWT_TYP
    );
    assert_eq!(
        <StatusValidatedRegistrationCertificate as jwt::JwtTyp>::TYP,
        jwt::jades_b_b::JADES_B_B_JWT_TYP
    );
}

#[test]
fn structurally_validated_payload_roundtrips_through_cbor() {
    let certificate = valid_payload()
        .validate_structure(&legal_person_access_certificate_subject(), validation_time())
        .unwrap();
    let mut encoded = Vec::new();
    ciborium::into_writer(&certificate, &mut encoded).unwrap();

    let decoded: UncheckedRegistrationCertificate = ciborium::from_reader(encoded.as_slice()).unwrap();
    let decoded = decoded
        .validate_structure(&legal_person_access_certificate_subject(), validation_time())
        .unwrap();
    assert_eq!(decoded, certificate);
}
