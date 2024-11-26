use wallet::errors::UriIdentificationError;
use wallet::UriType;

pub enum IdentifyUriResult {
    PidIssuance,
    Disclosure,
}

impl TryFrom<Result<UriType, UriIdentificationError>> for IdentifyUriResult {
    type Error = UriIdentificationError;

    fn try_from(value: Result<UriType, UriIdentificationError>) -> Result<Self, Self::Error> {
        match value {
            Ok(uri_type) => match uri_type {
                UriType::PidIssuance(_) => Ok(Self::PidIssuance),
                UriType::Disclosure(_) => Ok(Self::Disclosure),
            },
            Err(e) => Err(e),
        }
    }
}
