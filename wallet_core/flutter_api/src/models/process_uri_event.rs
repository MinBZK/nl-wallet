use crate::errors::FlutterApiError;

pub enum ProcessUriEvent {
    PidIssuance(PidIssuanceEvent),
    UnknownUri,
}

pub enum PidIssuanceEvent {
    Authenticating,
    Success,
    Error(String), // This string contains a JSON encoded FlutterApiError.
}

impl<T> From<T> for PidIssuanceEvent
where
    T: Into<FlutterApiError>,
{
    fn from(value: T) -> Self {
        Self::Error(value.into().to_json())
    }
}
