use crypto::CredentialEcdsaKey;
use wscd::keyfactory::KeyFactory;

use crate::errors::Result;

use super::super::Mdoc;

impl Mdoc {
    pub fn credential_key<K, KF>(&self, key_factory: &KF) -> Result<K>
    where
        K: CredentialEcdsaKey,
        KF: KeyFactory<Key = K>,
    {
        let public_key = (&self.mso.device_key_info.device_key).try_into()?;
        let key = key_factory.generate_existing(&self.private_key_id, public_key);

        Ok(key)
    }
}
