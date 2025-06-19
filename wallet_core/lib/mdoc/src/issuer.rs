use ciborium::value::Value;
use coset::Header;
use coset::HeaderBuilder;

#[cfg(any(test, feature = "test"))]
use crypto::keys::EcdsaKey;
#[cfg(any(test, feature = "test"))]
use crypto::server_keys::KeyPair;

use crate::iso::*;
use crate::utils::cose::COSE_X5CHAIN_HEADER_LABEL;

#[cfg(any(test, feature = "test"))]
use crate::Result;

impl IssuerSigned {
    pub fn create_unprotected_header(x5chain: Vec<u8>) -> Header {
        HeaderBuilder::new()
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(x5chain))
            .build()
    }

    #[cfg(any(test, feature = "test"))]
    pub async fn resign(&mut self, key: &KeyPair<impl EcdsaKey>) -> Result<()> {
        use crate::utils::cose::MdocCose;

        let mut mso = self.issuer_auth.dangerous_parse_unverified()?.0;

        // Update (fill) the issuer_uri to match the new key
        mso.issuer_uri = Some(key.certificate().san_dns_name_or_uris()?.into_first());

        self.issuer_auth = MdocCose::sign(&mso.into(), self.issuer_auth.0.unprotected.clone(), key, true).await?;

        Ok(())
    }
}
