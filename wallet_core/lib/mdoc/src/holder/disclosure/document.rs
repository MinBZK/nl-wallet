use crate::iso::disclosure::DeviceSigned;
use crate::iso::disclosure::Document;

use super::mdoc::DisclosureMdoc;

impl Document {
    pub fn new(disclosure_mdoc: DisclosureMdoc, device_signed: DeviceSigned) -> Self {
        Document {
            doc_type: disclosure_mdoc.doc_type,
            issuer_signed: disclosure_mdoc.issuer_signed,
            device_signed,
            errors: None,
        }
    }
}
