use wallet::InvocationUri;
use wallet::RedirectUri;
use wallet::UriType;
use wallet::errors::UriIdentificationError;

pub enum IdentifyUriResult {
    PidIssuance,
    PidRenewal,
    PinRecovery,
    Disclosure,
    DisclosureBasedIssuance,
    Transfer,
    CredentialOffer,
}

impl TryFrom<Result<UriType, UriIdentificationError>> for IdentifyUriResult {
    type Error = UriIdentificationError;

    fn try_from(value: Result<UriType, UriIdentificationError>) -> Result<Self, Self::Error> {
        match value {
            Ok(uri_type) => match uri_type {
                UriType::Redirect(RedirectUri::PidIssuance) => Ok(Self::PidIssuance),
                UriType::Redirect(RedirectUri::PidRenewal) => Ok(Self::PidRenewal),
                UriType::Redirect(RedirectUri::PinRecovery) => Ok(Self::PinRecovery),
                UriType::Invocation(InvocationUri::Disclosure) => Ok(Self::Disclosure),
                UriType::Invocation(InvocationUri::DisclosureBasedIssuance) => Ok(Self::DisclosureBasedIssuance),
                UriType::Invocation(InvocationUri::Transfer) => Ok(Self::Transfer),
                UriType::Invocation(InvocationUri::CredentialOffer) => Ok(Self::CredentialOffer),
            },
            Err(e) => Err(e),
        }
    }
}
