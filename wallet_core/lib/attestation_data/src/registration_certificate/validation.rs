use chrono::DateTime;
use chrono::Months;
use chrono::Utc;
use dcql::CredentialQueryFormat;
use language_tags::LanguageTag;
use url::Url;

use super::payload::Credential;
use super::payload::MultiLanguageStringSet;
use super::payload::UncheckedRegistrationCertificate;
use crate::x509::RelyingParty;

const WRPRC_POLICY_IDENTIFIER: &str = "0.4.0.19475.3.1";
pub(super) const SERVICE_PROVIDER_ENTITLEMENT: &str = "https://uri.etsi.org/19475/Entitlement/Service_Provider";

const ANNEX_A_2_ENTITLEMENTS: [&str; 10] = [
    SERVICE_PROVIDER_ENTITLEMENT,
    "https://uri.etsi.org/19475/Entitlement/QEAA_Provider",
    "https://uri.etsi.org/19475/Entitlement/Non_Q_EAA_Provider",
    "https://uri.etsi.org/19475/Entitlement/PUB_EAA_Provider",
    "https://uri.etsi.org/19475/Entitlement/PID_Provider",
    "https://uri.etsi.org/19475/Entitlement/QCert_for_ESeal_Provider",
    "https://uri.etsi.org/19475/Entitlement/QCert_for_ESig_Provider",
    "https://uri.etsi.org/19475/Entitlement/rQSealCDs_Provider",
    "https://uri.etsi.org/19475/Entitlement/rQSigCDs_Provider",
    "https://uri.etsi.org/19475/Entitlement/ESig_ESeal_Creation_Provider",
];

pub(super) const ANNEX_A_3_1_SUB_ENTITLEMENTS: [&str; 5] = [
    "https://uri.etsi.org/19475/SubEntitlement/psp/psp-as",
    "https://uri.etsi.org/19475/SubEntitlement/psp/psp-pi",
    "https://uri.etsi.org/19475/SubEntitlement/psp/psp-ai",
    "https://uri.etsi.org/19475/SubEntitlement/psp/psp-ic",
    "https://uri.etsi.org/19475/SubEntitlement/psp/unspecified",
];

/// A registration-certificate payload that passed all synchronous structural and direct-WRPAC-binding checks.
///
/// Header and signature validation are handled by PVW-5898 and PVW-5899. Verification of the referenced status list is
/// a separate asynchronous step; use [`Self::validate_status`] to perform it.
pub struct StructurallyValidatedRegistrationCertificate(UncheckedRegistrationCertificate);

impl StructurallyValidatedRegistrationCertificate {
    pub fn payload(&self) -> &UncheckedRegistrationCertificate {
        &self.0
    }
}

#[derive(Debug)]
pub enum SubjectType {
    LegalPerson,
    NaturalPerson,
}

#[derive(Debug, thiserror::Error)]
pub enum MultiLanguageStringSetValidationError {
    #[error("language tag at index {index} is not valid BCP 47: `{lang}`")]
    InvalidLanguageTag { index: usize, lang: String },
    #[error("localized value at index {index} must not be empty")]
    EmptyValue { index: usize },
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialSetValidationError {
    #[error("credential metadata at index {index} has an empty `doctype_value`")]
    EmptyDoctype { index: usize },
    #[error("credential metadata at index {index} has an empty `vct_values` entry at index {value_index}")]
    EmptyVctValue { index: usize, value_index: usize },
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationCertificateValidationError {
    #[error("registration certificate has no `id`")]
    MissingId,
    #[error("status must use the direct object shape with `idx` encoded as a numeric string")]
    InvalidStatusWireFormat,
    #[error("field `{field}` must not be empty")]
    EmptyField { field: &'static str },
    #[error("registration certificate contains both legal-person and natural-person subject fields")]
    AmbiguousSubjectType,
    #[error("registration certificate subject type cannot be determined")]
    UndeterminedSubjectType,
    #[error("access certificate has no `{field}` for a {subject_type:?}")]
    MissingAccessCertificateIdentifier {
        field: &'static str,
        subject_type: SubjectType,
    },
    #[error(
        "registration certificate subject type {registration:?} does not match access certificate subject type \
         {access:?}"
    )]
    AccessCertificateSubjectTypeMismatch {
        registration: SubjectType,
        access: SubjectType,
    },
    #[error("registration certificate `sub` does not match access certificate `{field}`")]
    SubjectIdentifierMismatch { field: &'static str },
    #[error("country must contain exactly two characters")]
    InvalidCountry,
    #[error("support URI must be a URL or an email address")]
    InvalidSupportUri,
    #[error("service description at index {index} is invalid: {source}")]
    InvalidServiceDescription {
        index: usize,
        #[source]
        source: MultiLanguageStringSetValidationError,
    },
    #[error("supervisory authority email address is invalid")]
    InvalidSupervisoryAuthorityEmail,
    #[error("supervisory authority phone number is invalid")]
    InvalidSupervisoryAuthorityPhone,
    #[error("entitlement at index {index} is not recognized: `{entitlement}`")]
    InvalidEntitlement { index: usize, entitlement: String },
    #[error("registration certificate must contain at least one Annex A.2 entitlement")]
    MissingAnnexA2Entitlement,
    #[error("Service_Provider entitlement requires `credentials`")]
    MissingServiceProviderCredentials,
    #[error("Service_Provider entitlement requires `purpose`")]
    MissingServiceProviderPurpose,
    #[error("multi-language field `{field}` is invalid: {source}")]
    InvalidMultiLanguageStringSet {
        field: &'static str,
        #[source]
        source: MultiLanguageStringSetValidationError,
    },
    #[error("credential set `{field}` is invalid: {source}")]
    InvalidCredentialSet {
        field: &'static str,
        #[source]
        source: CredentialSetValidationError,
    },
    #[error("registration certificate expired at {expiration}")]
    Expired { expiration: DateTime<Utc> },
    #[error("registration certificate expiration is later than 12 months after issuance")]
    ExpirationTooLate,
    #[error("policy_id does not contain the WRPRC policy identifier")]
    MissingPolicyIdentifier,
}

fn validate_multi_language_string_set(
    value: &MultiLanguageStringSet,
) -> Result<(), MultiLanguageStringSetValidationError> {
    for (index, localized) in value.iter().enumerate() {
        if LanguageTag::parse(&localized.lang).is_err() {
            return Err(MultiLanguageStringSetValidationError::InvalidLanguageTag {
                index,
                lang: localized.lang.clone(),
            });
        }
        if is_empty(&localized.value) {
            return Err(MultiLanguageStringSetValidationError::EmptyValue { index });
        }
    }

    Ok(())
}

fn validate_credential_set(value: &[Credential]) -> Result<(), CredentialSetValidationError> {
    for (index, credential) in value.iter().enumerate() {
        match &credential.format {
            CredentialQueryFormat::MsoMdoc { doctype_value } if is_empty(doctype_value) => {
                return Err(CredentialSetValidationError::EmptyDoctype { index });
            }
            CredentialQueryFormat::SdJwt { vct_values } => {
                if let Some(value_index) = vct_values.iter().position(|value| is_empty(value)) {
                    return Err(CredentialSetValidationError::EmptyVctValue { index, value_index });
                }
            }
            CredentialQueryFormat::MsoMdoc { .. } => {}
        }
    }

    Ok(())
}

impl UncheckedRegistrationCertificate {
    pub fn validate_structure(
        self,
        access_certificate_subject: &RelyingParty,
        now: DateTime<Utc>,
    ) -> Result<StructurallyValidatedRegistrationCertificate, RegistrationCertificateValidationError> {
        let id = self
            .id
            .as_deref()
            .ok_or(RegistrationCertificateValidationError::MissingId)?;
        validate_non_empty("id", id)?;
        if !self.status.has_normative_input_format() {
            return Err(RegistrationCertificateValidationError::InvalidStatusWireFormat);
        }
        validate_non_empty("sub", &self.sub)?;
        if let Some(name) = &self.name {
            validate_non_empty("name", name)?;
        }

        self.validate_subject(access_certificate_subject)?;

        if self.country.chars().count() != 2 {
            return Err(RegistrationCertificateValidationError::InvalidCountry);
        }
        if Url::parse(&self.support_uri).is_err() && !is_email_address(&self.support_uri) {
            return Err(RegistrationCertificateValidationError::InvalidSupportUri);
        }

        for (index, description) in self.srv_description.iter().enumerate() {
            validate_multi_language_string_set(description).map_err(|source| {
                RegistrationCertificateValidationError::InvalidServiceDescription { index, source }
            })?;
        }

        self.validate_supervisory_authority()?;
        let is_service_provider = self.validate_entitlements()?;

        if let Some(credentials) = &self.credentials {
            validate_credential_set(credentials).map_err(|source| {
                RegistrationCertificateValidationError::InvalidCredentialSet {
                    field: "credentials",
                    source,
                }
            })?;
        } else if is_service_provider {
            return Err(RegistrationCertificateValidationError::MissingServiceProviderCredentials);
        }

        if let Some(purpose) = &self.purpose {
            validate_multi_language_string_set(purpose).map_err(|source| {
                RegistrationCertificateValidationError::InvalidMultiLanguageStringSet {
                    field: "purpose",
                    source,
                }
            })?;
        } else if is_service_provider {
            return Err(RegistrationCertificateValidationError::MissingServiceProviderPurpose);
        }

        if let Some(intended_use_id) = &self.intended_use_id {
            validate_non_empty("intended_use_id", intended_use_id)?;
        }
        if let Some(provides_attestations) = &self.provides_attestations {
            validate_credential_set(provides_attestations).map_err(|source| {
                RegistrationCertificateValidationError::InvalidCredentialSet {
                    field: "provides_attestations",
                    source,
                }
            })?;
        }

        if let Some(expiration) = self.exp {
            if now >= expiration {
                return Err(RegistrationCertificateValidationError::Expired { expiration });
            }
            if self
                .iat
                .checked_add_months(Months::new(12))
                .is_none_or(|latest_expiration| expiration > latest_expiration)
            {
                return Err(RegistrationCertificateValidationError::ExpirationTooLate);
            }
        }

        if !self.policy_id.iter().any(|policy| policy == WRPRC_POLICY_IDENTIFIER) {
            return Err(RegistrationCertificateValidationError::MissingPolicyIdentifier);
        }

        Ok(StructurallyValidatedRegistrationCertificate(self))
    }

    fn validate_subject(
        &self,
        access_certificate_subject: &RelyingParty,
    ) -> Result<(), RegistrationCertificateValidationError> {
        let subject_type = match (self.sub_ln.as_deref(), self.sub_gn.as_deref(), self.sub_fn.as_deref()) {
            (Some(sub_ln), None, None) => {
                validate_non_empty("sub_ln", sub_ln)?;
                SubjectType::LegalPerson
            }
            (None, Some(sub_gn), Some(sub_fn)) => {
                validate_non_empty("sub_gn", sub_gn)?;
                validate_non_empty("sub_fn", sub_fn)?;
                SubjectType::NaturalPerson
            }
            (Some(_), Some(_), _) | (Some(_), _, Some(_)) => {
                return Err(RegistrationCertificateValidationError::AmbiguousSubjectType);
            }
            _ => return Err(RegistrationCertificateValidationError::UndeterminedSubjectType),
        };

        let (subject_field, access_identifier) = match (&subject_type, access_certificate_subject) {
            (
                SubjectType::LegalPerson,
                RelyingParty::LegalPerson {
                    organization_identifier,
                    ..
                },
            ) => ("organizationIdentifier", organization_identifier.as_str()),
            (SubjectType::NaturalPerson, RelyingParty::NaturalPerson { serial_number, .. }) => {
                ("serialNumber", serial_number.as_str())
            }
            (_, access_certificate_subject) => {
                let access = match access_certificate_subject {
                    RelyingParty::LegalPerson { .. } => SubjectType::LegalPerson,
                    RelyingParty::NaturalPerson { .. } => SubjectType::NaturalPerson,
                };
                return Err(
                    RegistrationCertificateValidationError::AccessCertificateSubjectTypeMismatch {
                        registration: subject_type,
                        access,
                    },
                );
            }
        };

        if is_empty(access_identifier) {
            return Err(
                RegistrationCertificateValidationError::MissingAccessCertificateIdentifier {
                    field: subject_field,
                    subject_type,
                },
            );
        }
        if self.sub != access_identifier {
            return Err(RegistrationCertificateValidationError::SubjectIdentifierMismatch { field: subject_field });
        }

        Ok(())
    }

    fn validate_supervisory_authority(&self) -> Result<(), RegistrationCertificateValidationError> {
        if self
            .supervisory_authority
            .email
            .as_ref()
            .is_some_and(|email| !is_email_address(email))
        {
            return Err(RegistrationCertificateValidationError::InvalidSupervisoryAuthorityEmail);
        }
        if self
            .supervisory_authority
            .phone
            .as_ref()
            .is_some_and(|phone| !is_phone_number(phone))
        {
            return Err(RegistrationCertificateValidationError::InvalidSupervisoryAuthorityPhone);
        }
        Ok(())
    }

    fn validate_entitlements(&self) -> Result<bool, RegistrationCertificateValidationError> {
        let mut has_annex_a_2_entitlement = false;
        let mut is_service_provider = false;
        for (index, entitlement) in self.entitlements.iter().enumerate() {
            let value = entitlement.as_str();
            let is_annex_a_2 = ANNEX_A_2_ENTITLEMENTS.contains(&value);
            let is_sub_entitlement = ANNEX_A_3_1_SUB_ENTITLEMENTS.contains(&value);
            if !is_annex_a_2 && !is_sub_entitlement {
                return Err(RegistrationCertificateValidationError::InvalidEntitlement {
                    index,
                    entitlement: value.to_string(),
                });
            }
            has_annex_a_2_entitlement |= is_annex_a_2;
            is_service_provider |= value == SERVICE_PROVIDER_ENTITLEMENT;
        }

        if !has_annex_a_2_entitlement {
            return Err(RegistrationCertificateValidationError::MissingAnnexA2Entitlement);
        }

        Ok(is_service_provider)
    }
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), RegistrationCertificateValidationError> {
    if is_empty(value) {
        return Err(RegistrationCertificateValidationError::EmptyField { field });
    }
    Ok(())
}

fn is_empty(value: &str) -> bool {
    value.trim().is_empty()
}

fn is_email_address(value: &str) -> bool {
    value.contains('@')
}

fn is_phone_number(value: &str) -> bool {
    let Some(number) = value.strip_prefix('+') else {
        return false;
    };
    number.chars().any(|character| character.is_ascii_digit())
        && number
            .chars()
            .all(|character| character.is_ascii_digit() || matches!(character, '-' | ' ' | '(' | ')'))
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use chrono::TimeZone;
    use chrono::Utc;
    use crypto::x509::DistinguishedName;
    use serde_json::json;
    use utils::vec_nonempty;

    use super::super::payload::Credential;
    use super::super::payload::MultiLanguageString;
    use super::super::payload::UncheckedRegistrationCertificate;
    use super::super::test::legal_person_access_certificate_subject;
    use super::super::test::valid_payload;
    use super::super::test::valid_payload_json;
    use super::super::test::validation_time;
    use super::ANNEX_A_3_1_SUB_ENTITLEMENTS;
    use super::CredentialSetValidationError;
    use super::MultiLanguageStringSetValidationError;
    use super::RegistrationCertificateValidationError;
    use super::SERVICE_PROVIDER_ENTITLEMENT;
    use super::SubjectType;
    use crate::x509::RelyingParty;

    #[test]
    fn validate_payload_based_on_annex_c_example() {
        let certificate = valid_payload()
            .validate_structure(&legal_person_access_certificate_subject(), validation_time())
            .unwrap();

        assert_eq!(certificate.payload().id.as_deref(), Some("wrprc-example-1"));
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
        assert_eq!(certificate.payload().sub_gn.as_deref(), Some("Jane"));
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
        let mut missing_credentials = valid_payload();
        missing_credentials.entitlements = vec_nonempty![SERVICE_PROVIDER_ENTITLEMENT.parse().unwrap()];
        missing_credentials.credentials = None;

        assert_matches!(
            missing_credentials.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
            Err(RegistrationCertificateValidationError::MissingServiceProviderCredentials)
        );

        let mut missing_purpose = valid_payload();
        missing_purpose.entitlements = vec_nonempty![SERVICE_PROVIDER_ENTITLEMENT.parse().unwrap()];
        missing_purpose.credentials = Some(Vec::new());
        missing_purpose.purpose = None;
        assert_matches!(
            missing_purpose.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
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
        let issued_at = Utc.with_ymd_and_hms(2024, 2, 29, 12, 0, 0).unwrap();
        let now = Utc.with_ymd_and_hms(2025, 2, 28, 11, 59, 59).unwrap();

        let mut valid = valid_payload();
        valid.iat = issued_at;
        valid.exp = Some(Utc.with_ymd_and_hms(2025, 2, 28, 12, 0, 0).unwrap());
        valid
            .validate_structure(&legal_person_access_certificate_subject(), now)
            .unwrap();

        let mut too_late = valid_payload();
        too_late.iat = issued_at;
        too_late.exp = Some(Utc.with_ymd_and_hms(2025, 2, 28, 12, 0, 1).unwrap());
        assert_matches!(
            too_late.validate_structure(&legal_person_access_certificate_subject(), now),
            Err(RegistrationCertificateValidationError::ExpirationTooLate)
        );

        let mut expired = valid_payload();
        expired.iat = issued_at;
        expired.exp = Some(now);
        assert_matches!(
            expired.validate_structure(&legal_person_access_certificate_subject(), now),
            Err(RegistrationCertificateValidationError::Expired { .. })
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
            let mut payload = valid_payload();
            payload.srv_description = vec_nonempty![vec_nonempty![MultiLanguageString {
                lang: language_tag.to_string(),
                value: "value".to_string(),
            }]];
            payload
                .validate_structure(&legal_person_access_certificate_subject(), validation_time())
                .unwrap();
        }

        for language_tag in ["", "e", "en_US", "en-", "en-US-abc", "en-a", "en-x", "en-ü"] {
            let mut payload = valid_payload();
            payload.srv_description = vec_nonempty![vec_nonempty![MultiLanguageString {
                lang: language_tag.to_string(),
                value: "value".to_string(),
            }]];
            assert_matches!(
                payload.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
                Err(RegistrationCertificateValidationError::InvalidServiceDescription {
                    source: MultiLanguageStringSetValidationError::InvalidLanguageTag { .. },
                    ..
                }),
                "{language_tag}"
            );
        }

        let mut payload = valid_payload();
        payload.srv_description = vec_nonempty![vec_nonempty![MultiLanguageString {
            lang: "en".to_string(),
            value: "  ".to_string(),
        }]];
        assert_matches!(
            payload.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
            Err(RegistrationCertificateValidationError::InvalidServiceDescription {
                source: MultiLanguageStringSetValidationError::EmptyValue { index: 0 },
                ..
            })
        );
    }

    #[test]
    fn validate_credential_metadata() {
        let mut payload = valid_payload();
        payload.credentials = Some(Vec::new());
        payload
            .validate_structure(&legal_person_access_certificate_subject(), validation_time())
            .unwrap();

        let empty_doctype: Credential = serde_json::from_value(json!({
            "format": "mso_mdoc",
            "meta": { "doctype_value": " " }
        }))
        .unwrap();
        let mut payload = valid_payload();
        payload.credentials = Some(vec![empty_doctype]);
        assert_matches!(
            payload.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
            Err(RegistrationCertificateValidationError::InvalidCredentialSet {
                source: CredentialSetValidationError::EmptyDoctype { index: 0 },
                ..
            })
        );

        let empty_vct: Credential = serde_json::from_value(json!({
            "format": "dc+sd-jwt",
            "meta": { "vct_values": ["urn:eudi:pid:1", " "] }
        }))
        .unwrap();
        let mut payload = valid_payload();
        payload.credentials = Some(vec![empty_vct]);
        assert_matches!(
            payload.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
            Err(RegistrationCertificateValidationError::InvalidCredentialSet {
                source: CredentialSetValidationError::EmptyVctValue {
                    index: 0,
                    value_index: 1,
                },
                ..
            })
        );
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
}
