use crypto::CredentialEcdsaKey;
use crypto::wscd::DisclosureWscd;

use crate::errors::Result;

use super::super::Mdoc;

impl Mdoc {
    pub fn credential_key<K, W>(&self, wscd: &W) -> Result<K>
    where
        K: CredentialEcdsaKey,
        W: DisclosureWscd<Key = K>,
    {
        let public_key = (&self.mso.device_key_info.device_key).try_into()?;
        let key = wscd.new_key(&self.private_key_id, public_key);

        Ok(key)
    }
}
