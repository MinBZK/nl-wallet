use url::Url;

use wallet::attestation_data::ReaderRegistration;
use wallet::errors::DisclosureError;
use wallet::openid4vc::SessionType;
use wallet::DisclosureProposal;

use super::attestation::AttestationPresentation;
use super::image::Image;
use super::instruction::WalletInstructionError;
use super::localize::LocalizedString;

pub struct Organization {
    pub legal_name: Vec<LocalizedString>,
    pub display_name: Vec<LocalizedString>,
    pub description: Vec<LocalizedString>,
    pub image: Option<Image>,
    pub web_url: Option<String>,
    pub privacy_policy_url: Option<String>,
    pub kvk: Option<String>,
    pub city: Option<Vec<LocalizedString>>,
    pub category: Vec<LocalizedString>,
    pub department: Option<Vec<LocalizedString>>,
    pub country_code: Option<String>,
}

pub struct RequestPolicy {
    pub data_storage_duration_in_minutes: Option<u64>,
    pub data_shared_with_third_parties: bool,
    pub data_deletion_possible: bool,
    pub policy_url: String,
}

pub struct MissingAttribute {
    pub labels: Vec<LocalizedString>,
}

pub enum DisclosureType {
    Login,
    Regular,
}

pub enum DisclosureSessionType {
    SameDevice,
    CrossDevice,
}

impl From<SessionType> for DisclosureSessionType {
    fn from(source: SessionType) -> Self {
        match source {
            SessionType::SameDevice => Self::SameDevice,
            SessionType::CrossDevice => Self::CrossDevice,
        }
    }
}

pub enum StartDisclosureResult {
    Request {
        relying_party: Organization,
        policy: RequestPolicy,
        requested_attestations: Vec<AttestationPresentation>,
        shared_data_with_relying_party_before: bool,
        session_type: DisclosureSessionType,
        request_purpose: Vec<LocalizedString>,
        request_origin_base_url: String,
        request_type: DisclosureType,
    },
    RequestAttributesMissing {
        relying_party: Organization,
        missing_attributes: Vec<MissingAttribute>,
        shared_data_with_relying_party_before: bool,
        session_type: DisclosureSessionType,
        request_purpose: Vec<LocalizedString>,
        request_origin_base_url: String,
    },
}

pub enum AcceptDisclosureResult {
    Ok { return_url: Option<String> },
    InstructionError { error: WalletInstructionError },
}

pub struct RPLocalizedStrings(pub wallet::attestation_data::LocalizedStrings);

impl From<RPLocalizedStrings> for Vec<LocalizedString> {
    fn from(value: RPLocalizedStrings) -> Self {
        let RPLocalizedStrings(wallet::attestation_data::LocalizedStrings(localized_strings)) = value;
        localized_strings
            .iter()
            .map(|(language, value)| LocalizedString {
                language: language.to_owned(),
                value: value.to_owned(),
            })
            .collect()
    }
}

impl From<wallet::attestation_data::Image> for Image {
    fn from(value: wallet::attestation_data::Image) -> Self {
        match value {
            wallet::attestation_data::Image::Svg(xml) => Image::Svg { xml },
            wallet::attestation_data::Image::Png(data) => Image::Png { data },
            wallet::attestation_data::Image::Jpeg(data) => Image::Jpeg { data },
        }
    }
}

impl From<wallet::attestation_data::Organization> for Organization {
    fn from(value: wallet::attestation_data::Organization) -> Self {
        Organization {
            legal_name: RPLocalizedStrings(value.legal_name).into(),
            display_name: RPLocalizedStrings(value.display_name).into(),
            description: RPLocalizedStrings(value.description).into(),
            image: value.logo.map(|logo| logo.into()),
            kvk: value.kvk,
            city: value.city.map(|city| RPLocalizedStrings(city).into()),
            category: RPLocalizedStrings(value.category).into(),
            department: value.department.map(|department| RPLocalizedStrings(department).into()),
            country_code: value.country_code,
            web_url: value.web_url.map(|url| url.to_string()),
            privacy_policy_url: value.privacy_policy_url.map(|url| url.to_string()),
        }
    }
}

impl From<&ReaderRegistration> for RequestPolicy {
    fn from(value: &ReaderRegistration) -> Self {
        let data_storage_duration_in_minutes = value
            .retention_policy
            .intent_to_retain
            .then_some(value.retention_policy.max_duration_in_minutes)
            .flatten();
        RequestPolicy {
            data_storage_duration_in_minutes,
            data_shared_with_third_parties: value.sharing_policy.intent_to_share,
            data_deletion_possible: value.deletion_policy.deleteable,
            policy_url: value
                .organization
                .privacy_policy_url
                .as_ref()
                .map(|url| url.to_string())
                .unwrap_or_default(),
        }
    }
}

// TODO (PVW-3813): Actually translate the missing attributes using the TAS cache.
impl From<String> for MissingAttribute {
    fn from(value: String) -> Self {
        const LANGUAGES: &[&str] = &["nl", "en"];

        let labels = LANGUAGES
            .iter()
            .zip(itertools::repeat_n(value, LANGUAGES.len()))
            .map(|(language, value)| LocalizedString {
                language: language.to_string(),
                value,
            })
            .collect();

        Self { labels }
    }
}

impl From<wallet::DisclosureType> for DisclosureType {
    fn from(source: wallet::DisclosureType) -> Self {
        match source {
            wallet::DisclosureType::Login => DisclosureType::Login,
            wallet::DisclosureType::Regular => DisclosureType::Regular,
        }
    }
}

impl TryFrom<Result<DisclosureProposal, DisclosureError>> for StartDisclosureResult {
    type Error = DisclosureError;

    fn try_from(value: Result<DisclosureProposal, DisclosureError>) -> Result<Self, Self::Error> {
        match value {
            Ok(proposal) => {
                let policy: RequestPolicy = (&proposal.reader_registration).into();
                let request_purpose: Vec<LocalizedString> =
                    RPLocalizedStrings(proposal.reader_registration.purpose_statement).into();
                let result = StartDisclosureResult::Request {
                    relying_party: proposal.reader_registration.organization.into(),
                    policy,
                    requested_attestations: proposal
                        .attestations
                        .into_iter()
                        .map(AttestationPresentation::from)
                        .collect(),
                    shared_data_with_relying_party_before: proposal.shared_data_with_relying_party_before,
                    session_type: proposal.session_type.into(),
                    request_purpose,
                    request_origin_base_url: proposal.reader_registration.request_origin_base_url.into(),
                    request_type: proposal.disclosure_type.into(),
                };

                Ok(result)
            }
            Err(error) => match error {
                DisclosureError::AttributesNotAvailable {
                    reader_registration,
                    missing_attributes,
                    shared_data_with_relying_party_before,
                    session_type,
                } => {
                    let request_purpose: Vec<LocalizedString> =
                        RPLocalizedStrings(reader_registration.purpose_statement).into();
                    let missing_attributes = missing_attributes.into_iter().map(MissingAttribute::from).collect();
                    let result = StartDisclosureResult::RequestAttributesMissing {
                        relying_party: reader_registration.organization.into(),
                        missing_attributes,
                        shared_data_with_relying_party_before,
                        session_type: session_type.into(),
                        request_purpose,
                        request_origin_base_url: reader_registration.request_origin_base_url.into(),
                    };

                    Ok(result)
                }
                _ => Err(error),
            },
        }
    }
}

impl TryFrom<Result<Option<Url>, DisclosureError>> for AcceptDisclosureResult {
    type Error = DisclosureError;

    fn try_from(value: Result<Option<Url>, DisclosureError>) -> Result<Self, Self::Error> {
        match value {
            Ok(return_url) => Ok(AcceptDisclosureResult::Ok {
                return_url: return_url.map(|return_url| return_url.into()),
            }),
            Err(DisclosureError::Instruction(instruction_error)) => Ok(AcceptDisclosureResult::InstructionError {
                error: instruction_error.try_into().map_err(DisclosureError::Instruction)?,
            }),
            Err(error) => Err(error),
        }
    }
}
