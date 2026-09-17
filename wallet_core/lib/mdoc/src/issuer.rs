#[cfg(any(test, feature = "test"))]
use crypto::keys::EcdsaKey;
#[cfg(any(test, feature = "test"))]
use crypto::server_keys::KeyPair;

#[cfg(any(test, feature = "test"))]
use crate::Result;
use crate::iso::*;

impl IssuerSigned {
    #[cfg(any(test, feature = "test"))]
    pub async fn resign(&mut self, key: &KeyPair<impl EcdsaKey>) -> Result<()> {
        use crate::utils::cose::TypedCose;

        let mso = self.issuer_auth.dangerous_parse_unverified()?.0;

        self.issuer_auth =
            TypedCose::sign(&mso.into(), self.issuer_auth.as_ref().unprotected.clone(), key, true).await?;

        Ok(())
    }
}
