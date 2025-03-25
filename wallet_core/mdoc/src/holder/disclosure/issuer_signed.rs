use p256::ecdsa::VerifyingKey;

use crate::errors::Result;
use crate::iso::disclosure::IssuerSigned;

impl IssuerSigned {
    pub fn public_key(&self) -> Result<VerifyingKey> {
        let public_key = self
            .issuer_auth
            .dangerous_parse_unverified()?
            .0
            .device_key_info
            .try_into()?;
        Ok(public_key)
    }
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use crypto::mock_remote::MockRemoteEcdsaKey;

    use crate::holder::Mdoc;

    #[tokio::test]
    async fn test_issuer_signed_public_key() {
        let key = SigningKey::random(&mut OsRng);
        let key = MockRemoteEcdsaKey::new("identifier".to_string(), key);
        let mdoc = Mdoc::new_mock_with_key(&key).await;

        let public_key = mdoc
            .issuer_signed
            .public_key()
            .expect("Could not get public key from from IssuerSigned");

        // The example mdoc should contain the generated key.
        assert_eq!(public_key, *key.verifying_key());
    }
}
