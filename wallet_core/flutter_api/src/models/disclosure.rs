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
