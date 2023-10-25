use super::card::{CardAttribute, LocalizedString};

pub struct RelyingParty {
    pub name: String,
    // TODO: add more values.
}

pub struct MissingAttribute {
    pub labels: Vec<LocalizedString>,
}

pub struct RequestedCard {
    pub doc_type: String,
    pub attributes: Vec<CardAttribute>,
}

pub enum DisclosureResult {
    Request {
        relying_party: RelyingParty,
        requested_cards: Vec<RequestedCard>,
    },
    RequestAttributesMissing {
        relying_party: RelyingParty,
        missing_attributes: Vec<MissingAttribute>,
    },
}
