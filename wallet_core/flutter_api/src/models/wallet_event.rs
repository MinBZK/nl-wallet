use crate::models::card::{Card, LocalizedString};
use crate::models::disclosure::{Organization, RequestPolicy, RequestedCard};

pub enum WalletEvent {
    Disclosure {
        //ISO8601
        date_time: String,
        relying_party: Organization,
        purpose: Vec<LocalizedString>,
        requested_cards: Vec<RequestedCard>,
        request_policy: RequestPolicy,
        status: DisclosureStatus,
    },
    Issuance {
        //ISO8601
        date_time: String,
        issuer: Organization,
        card: Card,
    },
}

pub enum DisclosureStatus {
    Success,
    Cancelled,
    Error,
}
