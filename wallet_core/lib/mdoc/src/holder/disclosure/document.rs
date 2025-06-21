use crate::iso::disclosure::DeviceSigned;
use crate::iso::disclosure::Document;

use super::super::Mdoc;

impl Document {
    pub fn new(mdoc: Mdoc, device_signed: DeviceSigned) -> Self {
        Document {
            doc_type: mdoc.mso.doc_type,
            issuer_signed: mdoc.issuer_signed,
            device_signed,
            errors: None,
        }
    }
}
