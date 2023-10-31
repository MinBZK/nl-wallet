use crate::{DataElementIdentifier, DocType, NameSpace};

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct AttributeIdentifier {
    pub doc_type: DocType,
    pub namespace: NameSpace,
    pub attribute: DataElementIdentifier,
}

impl std::fmt::Debug for AttributeIdentifier {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_fmt(format_args!("{}/{}/{}", self.doc_type, self.namespace, self.attribute))
    }
}
