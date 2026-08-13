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

        for (index, description) in self.srv_description.0.iter().enumerate() {
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
