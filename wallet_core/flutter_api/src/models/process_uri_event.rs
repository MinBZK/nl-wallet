use crate::errors::FlutterApiError;

use super::{
    card::Card,
    disclosure::{MissingAttribute, RelyingParty, RequestedCard},
};

pub enum ProcessUriEvent {
    PidIssuance { event: PidIssuanceEvent },
    Disclosure { event: DisclosureEvent },
    UnknownUri,
}

pub enum PidIssuanceEvent {
    Authenticating,
    Success { preview_cards: Vec<Card> },
    Error { data: String }, // This string contains a JSON encoded FlutterApiError.
}

pub enum DisclosureEvent {
    FetchingRequest,
    Request {
        relying_party: RelyingParty,
        requested_cards: Vec<RequestedCard>,
    },
    RequestAttributesMissing {
        relying_party: RelyingParty,
        missing_attributes: Vec<MissingAttribute>,
    },
    Error {
        data: String,
    }, // This string contains a JSON encoded FlutterApiError.
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
impl<T> From<T> for DisclosureEvent
where
    T: Into<FlutterApiError>,
{
    fn from(value: T) -> Self {
        Self::Error {
            data: value.into().to_json(),
        }
    }
}
