use indexmap::IndexMap;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

use nl_wallet_mdoc::{
    basic_sa_ext::Entry,
    holder::{Mdoc, ProposedAttributes, ProposedDocumentAttributes},
    utils::{cose::CoseError, x509::Certificate},
    DataElementIdentifier, DataElementValue, DocType, NameSpace,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryEventAttributes {
    pub issuer: Certificate,
    pub attributes: IndexMap<NameSpace, IndexMap<DataElementIdentifier, DataElementValue>>,
}

impl From<(Certificate, IndexMap<NameSpace, Vec<Entry>>)> for HistoryEventAttributes {
    fn from((issuer, attributes): (Certificate, IndexMap<NameSpace, Vec<Entry>>)) -> Self {
        Self {
            issuer,
            attributes: attributes
                .into_iter()
                .map(|(namespace, attributes)| {
                    (
                        namespace,
                        attributes.into_iter().map(|entry| (entry.name, entry.value)).collect(),
                    )
                })
                .collect(),
        }
    }
}

impl From<HistoryEventAttributes> for IndexMap<NameSpace, Vec<Entry>> {
    fn from(source: HistoryEventAttributes) -> IndexMap<NameSpace, Vec<Entry>> {
        source
            .attributes
            .into_iter()
            .map(|(namespace, attributes)| {
                (
                    namespace,
                    attributes
                        .into_iter()
                        .map(|(name, value)| Entry { name, value })
                        .collect(),
                )
            })
            .collect()
    }
}

impl From<ProposedDocumentAttributes> for HistoryEventAttributes {
    fn from(source: ProposedDocumentAttributes) -> Self {
        (source.issuer, source.attributes).into()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, FromJsonQueryResult, PartialEq)]
pub struct HistoryEventDocuments(pub IndexMap<DocType, HistoryEventAttributes>);

impl TryFrom<Vec<Mdoc>> for HistoryEventDocuments {
    type Error = CoseError;
    fn try_from(source: Vec<Mdoc>) -> Result<Self, Self::Error> {
        let doc_type_map = source
            .into_iter()
            .map(|mdoc| {
                let doc_type = mdoc.doc_type.clone();
                let issuer = mdoc.issuer_certificate()?;
                let attributes = mdoc.attributes();
                Ok((doc_type, (issuer, attributes).into()))
            })
            .collect::<Result<IndexMap<_, _>, CoseError>>()?;
        Ok(Self(doc_type_map))
    }
}

impl From<ProposedAttributes> for HistoryEventDocuments {
    fn from(source: ProposedAttributes) -> Self {
        let documents = source
            .into_iter()
            .map(|(doc_type, document)| (doc_type, document.into()))
            .collect();
        Self(documents)
    }
}
