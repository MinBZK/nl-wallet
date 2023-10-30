use wallet::{errors::DisclosureError, mdoc::ReaderRegistration, MissingDisclosureAttributes};

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

impl From<ReaderRegistration> for RelyingParty {
    fn from(value: ReaderRegistration) -> Self {
        // TODO: Implement proper conversion from `ReaderRegistration` with more fields.
        RelyingParty {
            name: value.name.0.into_values().next().unwrap(),
        }
    }
}

impl MissingAttribute {
    fn attributes_from_missing_disclosure_attributes(attributes: Vec<MissingDisclosureAttributes>) -> Vec<Self> {
        attributes
            .into_iter()
            .flat_map(|doc_attributes| doc_attributes.attributes.into_iter())
            .map(|(_, labels)| {
                let labels = labels
                    .into_iter()
                    .map(|(language, value)| LocalizedString {
                        language: language.to_string(),
                        value: value.to_string(),
                    })
                    .collect::<Vec<_>>();

                MissingAttribute { labels }
            })
            .collect::<Vec<_>>()
    }
}

impl TryFrom<Result<(), DisclosureError>> for DisclosureResult {
    type Error = DisclosureError;

    fn try_from(value: Result<(), DisclosureError>) -> Result<Self, Self::Error> {
        match value {
            Ok(_) => {
                // TODO: use actual ok result
                let result = DisclosureResult::Request {
                    relying_party: RelyingParty { name: "".to_string() },
                    requested_cards: Default::default(),
                };

                Ok(result)
            }
            Err(error) => match error {
                DisclosureError::AttributesNotAvailable {
                    reader_registration,
                    missing_attributes,
                } => {
                    let missing_attributes =
                        MissingAttribute::attributes_from_missing_disclosure_attributes(missing_attributes);
                    let result = DisclosureResult::RequestAttributesMissing {
                        relying_party: (*reader_registration).into(),
                        missing_attributes,
                    };

                    Ok(result)
                }
                _ => Err(error),
            },
        }
    }
}
