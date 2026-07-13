use attestation_types::status_claim::StatusClaim;
use attestation_types::status_claim::StatusListClaim;
use chrono::DateTime;
use chrono::Months;
use chrono::Utc;
use dcql::ClaimsQuery;
use dcql::CredentialQueryFormat;
use language_tags::LanguageTag;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de;
use serde::ser::SerializeStruct;
use serde_with::skip_serializing_none;
use url::Url;
use utils::vec_at_least::VecNonEmpty;

use crate::x509::RelyingParty;

pub const WRPRC_POLICY_IDENTIFIER: &str = "0.4.0.19475.3.1";
pub const SERVICE_PROVIDER_ENTITLEMENT: &str = "https://uri.etsi.org/19475/Entitlement/Service_Provider";

pub const ANNEX_A_2_ENTITLEMENTS: [&str; 10] = [
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

pub const ANNEX_A_3_1_SUB_ENTITLEMENTS: [&str; 5] = [
    "https://uri.etsi.org/19475/SubEntitlement/psp/psp-as",
    "https://uri.etsi.org/19475/SubEntitlement/psp/psp-pi",
    "https://uri.etsi.org/19475/SubEntitlement/psp/psp-ai",
    "https://uri.etsi.org/19475/SubEntitlement/psp/psp-ic",
    "https://uri.etsi.org/19475/SubEntitlement/psp/unspecified",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiLanguageString {
    pub lang: String,
    #[serde(alias = "content")]
    pub value: String,
}

pub type MultiLanguageStringSet = VecNonEmpty<MultiLanguageString>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum ServiceDescriptionsWireFormat {
    Nested(VecNonEmpty<MultiLanguageStringSet>),
    Flat(MultiLanguageStringSet),
}

/// Localized descriptions of one or more services.
///
/// ETSI TS 119 475 Annex B models this as an array of arrays, while the validation table in PVW-5867 describes a
/// single array of objects. Both forms are accepted and normalized to the nested ETSI representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceDescriptions(pub VecNonEmpty<MultiLanguageStringSet>);

impl Serialize for ServiceDescriptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ServiceDescriptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let descriptions = ServiceDescriptionsWireFormat::deserialize(deserializer)?;
        Ok(Self(match descriptions {
            ServiceDescriptionsWireFormat::Nested(descriptions) => descriptions,
            ServiceDescriptionsWireFormat::Flat(description) => utils::vec_nonempty![description],
        }))
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
    #[serde(flatten)]
    pub format: CredentialQueryFormat,
    pub claim: Option<Vec<ClaimsQuery>>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupervisoryAuthority {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub uri: Option<Url>,
}

/// Reference to the status list entry for this registration certificate.
///
/// The direct `{ "idx": "0", "uri": "..." }` shape required by ETSI TS 119 475 clause 6.2.6.1 is emitted.
/// Deserialization also accepts the nested OAuth Status List shape used by the informative Annex C example, but
/// structural validation only accepts the normative direct shape with a numeric-string index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationCertificateStatus {
    status_claim: StatusClaim,
    input_format: RegistrationCertificateStatusInputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistrationCertificateStatusInputFormat {
    DirectNumericString,
    DirectInteger,
    NestedStatusList,
}

impl RegistrationCertificateStatus {
    pub fn status_claim(&self) -> &StatusClaim {
        &self.status_claim
    }

    pub fn status_list_claim(&self) -> &StatusListClaim {
        let StatusClaim::StatusList(status_list_claim) = &self.status_claim;
        status_list_claim
    }

    fn has_normative_input_format(&self) -> bool {
        self.input_format == RegistrationCertificateStatusInputFormat::DirectNumericString
    }
}

impl From<StatusListClaim> for RegistrationCertificateStatus {
    fn from(value: StatusListClaim) -> Self {
        Self {
            status_claim: StatusClaim::StatusList(value),
            input_format: RegistrationCertificateStatusInputFormat::DirectNumericString,
        }
    }
}

impl From<StatusClaim> for RegistrationCertificateStatus {
    fn from(value: StatusClaim) -> Self {
        Self {
            status_claim: value,
            input_format: RegistrationCertificateStatusInputFormat::DirectNumericString,
        }
    }
}

impl From<RegistrationCertificateStatus> for StatusClaim {
    fn from(value: RegistrationCertificateStatus) -> Self {
        value.status_claim
    }
}

impl From<RegistrationCertificateStatus> for StatusListClaim {
    fn from(value: RegistrationCertificateStatus) -> Self {
        let StatusClaim::StatusList(status_list_claim) = value.status_claim;
        status_list_claim
    }
}

impl Serialize for RegistrationCertificateStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let status_list_claim = self.status_list_claim();
        let mut status = serializer.serialize_struct("RegistrationCertificateStatus", 2)?;
        status.serialize_field("idx", &status_list_claim.idx.to_string())?;
        status.serialize_field("uri", &status_list_claim.uri)?;
        status.end()
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RegistrationCertificateStatusWireFormat {
    NestedStatusList(StatusClaim),
    Direct { idx: StatusListIndex, uri: Url },
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StatusListIndex {
    String(String),
    Integer(u32),
}

fn parse_status_list_index<E>(value: &str) -> Result<u32, E>
where
    E: de::Error,
{
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(E::custom(format!(
            "status list index `{value}` is not a numeric string"
        )));
    }

    value.parse().map_err(|_| {
        E::custom(format!(
            "status list index `{value}` does not fit in a 32-bit unsigned integer"
        ))
    })
}

impl<'de> Deserialize<'de> for RegistrationCertificateStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let status = RegistrationCertificateStatusWireFormat::deserialize(deserializer)?;
        let (status_claim, input_format) = match status {
            RegistrationCertificateStatusWireFormat::NestedStatusList(status_claim) => {
                (status_claim, RegistrationCertificateStatusInputFormat::NestedStatusList)
            }
            RegistrationCertificateStatusWireFormat::Direct { idx, uri } => {
                let (idx, input_format) = match idx {
                    StatusListIndex::String(value) => (
                        parse_status_list_index(&value)?,
                        RegistrationCertificateStatusInputFormat::DirectNumericString,
                    ),
                    StatusListIndex::Integer(value) => (value, RegistrationCertificateStatusInputFormat::DirectInteger),
                };
                (StatusClaim::StatusList(StatusListClaim { idx, uri }), input_format)
            }
        };
        Ok(Self {
            status_claim,
            input_format,
        })
    }
}

/// Intermediary data is parsed so it is not discarded, but its WRPAC binding is deliberately left to PVW-6062.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intermediary {
    pub sub: String,
    #[serde(alias = "name")]
    pub sname: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UncheckedRegistrationCertificate {
    /// Optional at parse time because the informative Annex C example omits it. Validation always requires it.
    pub id: Option<String>,
    pub name: Option<String>,
    pub sub: String,
    pub sub_ln: Option<String>,
    pub sub_gn: Option<String>,
    pub sub_fn: Option<String>,
    pub country: String,
    pub registry_uri: Url,
    pub support_uri: String,
    pub info_uri: Option<Url>,
    pub privacy_policy: Option<Url>,
    pub srv_description: ServiceDescriptions,
    pub supervisory_authority: SupervisoryAuthority,
    pub entitlements: VecNonEmpty<Url>,
    pub credentials: Option<Vec<Credential>>,
    pub purpose: Option<MultiLanguageStringSet>,
    pub intended_use_id: Option<String>,
    pub provides_attestations: Option<Vec<Credential>>,
    pub public_body: Option<bool>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub iat: DateTime<Utc>,
    #[serde(
        default,
        with = "chrono::serde::ts_seconds_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub exp: Option<DateTime<Utc>>,
    pub status: RegistrationCertificateStatus,
    pub policy_id: Vec<String>,
    pub certificate_policy: Url,
    pub intermediary: Option<Intermediary>,
}

/// A registration-certificate payload that passed all synchronous structural and direct-WRPAC-binding checks.
///
/// Header, signature, and referenced status-list validation are handled separately.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct StructurallyValidatedRegistrationCertificate(UncheckedRegistrationCertificate);

impl StructurallyValidatedRegistrationCertificate {
    pub fn payload(&self) -> &UncheckedRegistrationCertificate {
        &self.0
    }

    pub fn into_payload(self) -> UncheckedRegistrationCertificate {
        self.0
    }

    pub fn id(&self) -> &str {
        self.0
            .id
            .as_deref()
            .expect("a structurally validated registration certificate always has an id")
    }

    pub fn subject_type(&self) -> SubjectType {
        if self.0.sub_ln.is_some() {
            SubjectType::LegalPerson
        } else {
            SubjectType::NaturalPerson
        }
    }

    pub fn status(&self) -> &RegistrationCertificateStatus {
        &self.0.status
    }
}

impl AsRef<UncheckedRegistrationCertificate> for StructurallyValidatedRegistrationCertificate {
    fn as_ref(&self) -> &UncheckedRegistrationCertificate {
        self.payload()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub fn validate_multi_language_string_set(
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

pub fn validate_credential_set(value: &[Credential]) -> Result<(), CredentialSetValidationError> {
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
    ) -> Result<SubjectType, RegistrationCertificateValidationError> {
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

        let (subject_field, access_identifier) = match (subject_type, access_certificate_subject) {
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
            (registration, access_certificate_subject) => {
                let access = match access_certificate_subject {
                    RelyingParty::LegalPerson { .. } => SubjectType::LegalPerson,
                    RelyingParty::NaturalPerson { .. } => SubjectType::NaturalPerson,
                };
                return Err(
                    RegistrationCertificateValidationError::AccessCertificateSubjectTypeMismatch {
                        registration,
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

        Ok(subject_type)
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

impl jwt::JwtTyp for UncheckedRegistrationCertificate {
    const TYP: &'static str = jwt::jades_b_b::JADES_B_B_JWT_TYP;
}

impl jwt::JwtTyp for StructurallyValidatedRegistrationCertificate {
    const TYP: &'static str = jwt::jades_b_b::JADES_B_B_JWT_TYP;
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
mod tests;
