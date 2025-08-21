use crate::iso::disclosure::DeviceSigned;
use crate::iso::disclosure::Document;

use super::mdoc::PartialMdoc;

impl Document {
    pub fn new(partial_mdoc: PartialMdoc, device_signed: DeviceSigned) -> Self {
        Document {
            doc_type: partial_mdoc.doc_type,
            issuer_signed: partial_mdoc.issuer_signed,
            device_signed,
            errors: None,
        }
    }
}
