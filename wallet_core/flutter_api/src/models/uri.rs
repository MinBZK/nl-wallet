use wallet::UriType;
use wallet::errors::UriIdentificationError;

pub enum IdentifyUriResult {
    PidIssuance,
    Disclosure,
    DisclosureBasedIssuance,
}

impl TryFrom<Result<UriType, UriIdentificationError>> for IdentifyUriResult {
    type Error = UriIdentificationError;

    fn try_from(value: Result<UriType, UriIdentificationError>) -> Result<Self, Self::Error> {
        match value {
            Ok(uri_type) => match uri_type {
                UriType::PidIssuance => Ok(Self::PidIssuance),
                UriType::Disclosure => Ok(Self::Disclosure),
                UriType::DisclosureBasedIssuance => Ok(Self::DisclosureBasedIssuance),
            },
            Err(e) => Err(e),
        }
    }
}
