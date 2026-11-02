use std::convert::Infallible;
use std::sync::LazyLock;
use std::time::Duration;

use crypto::PublicKey;
use crypto::PublicKeyRef;
use crypto::p256_der::DerVerifyingKey;
use jwt::Algorithm;
use jwt::JwtValidation;
use jwt::PublicKeyByKid;

/// Leeway allowed when evaluating the `exp` of a [`WalletConfiguration`], in order to handle clock skew between the
/// wallet device and the configuration server.
///
/// [`WalletConfiguration`]: crate::wallet_config::WalletConfiguration
pub const CONFIG_EXPIRY_LEEWAY: Duration = Duration::from_secs(60);

pub static WALLET_CONFIG_VALIDATION: LazyLock<JwtValidation> = LazyLock::new(|| {
    let mut validation = JwtValidation::default_with_algorithms([Algorithm::ES256]);
    validation.require_exp();
    validation.set_leeway(CONFIG_EXPIRY_LEEWAY);

    validation
});

#[derive(Debug, Clone)]
pub struct ConfigKeyWithKid {
    pub kid: String,
    pub key: PublicKey,
}

impl ConfigKeyWithKid {
    pub fn new(kid: String, key: DerVerifyingKey) -> Self {
        let key = PublicKey::from(key.into_inner());
        Self { kid, key }
    }
}

impl<'a> PublicKeyByKid<'a> for ConfigKeyWithKid {
    type Key = PublicKeyRef<'a>;
    type Error = Infallible;

    fn find(&'a self, kid: &str) -> Result<Option<Self::Key>, Self::Error> {
        if kid == self.kid {
            Ok(Some((&self.key).into()))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {

    use std::assert_matches;

    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::Generate;

    use super::*;

    #[test]
    fn test_config_key_with_kid() {
        let kid = "v1".to_string();

        let keypair = SigningKey::generate();
        let key = DerVerifyingKey::from(*keypair.verifying_key());

        let config_key = ConfigKeyWithKid::new(kid, key.clone());
        assert_matches!(config_key.find("v1").unwrap(), Some(PublicKeyRef::ESP256(found_key)) if found_key == key.as_inner());
        assert!(config_key.find("v2").unwrap().is_none());
    }
}
