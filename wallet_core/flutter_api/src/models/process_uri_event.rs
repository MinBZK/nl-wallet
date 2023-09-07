use crate::errors::FlutterApiError;

use super::card::Card;

pub enum ProcessUriEvent {
    PidIssuance { event: PidIssuanceEvent },
    UnknownUri,
}

pub enum PidIssuanceEvent {
    Authenticating,
    Success { preview_cards: Vec<Card> },
    Error { data: String }, // This string contains a JSON encoded FlutterApiError.
}

impl<T> From<T> for PidIssuanceEvent
where
    T: Into<FlutterApiError>,
{
    fn from(value: T) -> Self {
        Self::Error {
            data: value.into().to_json(),
        }
    }
}
