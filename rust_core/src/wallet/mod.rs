pub mod pin;
pub mod pin_key;
pub mod signed;

use crate::{
    jwt::EcdsaDecodingKey,
    wp::{instructions, AccountServerClient, WalletCertificate},
};

use anyhow::{anyhow, Result};
use p256::ecdsa::{signature::Signer, Signature, VerifyingKey};

use self::pin_key::new_pin_salt;

/// Handle to a hardware-bound ECDSA private key.
pub trait HWBoundSigningKey: Signer<Signature> {
    fn verifying_key(&self) -> &VerifyingKey;
}

pub struct Wallet<T, S> {
    account_server: T,
    account_server_pubkey: Vec<u8>,
    registration_cert: Option<WalletCertificate>,

    pin_salt: Vec<u8>,
    hw_privkey: S,
}

impl<T, S> Wallet<T, S>
where
    T: AccountServerClient,
    S: HWBoundSigningKey,
{
    pub fn new(account_server: T, account_server_pubkey: Vec<u8>, hw_privkey: S) -> Wallet<T, S> {
        Wallet {
            account_server,
            account_server_pubkey,
            registration_cert: None,
            pin_salt: new_pin_salt(), // TODO look up in storage
            hw_privkey,
        }
    }

    pub fn register(&mut self, pin: String) -> Result<()> {
        let challenge = self.account_server.registration_challenge()?;

        let registration_message =
            instructions::Registration::new_signed(&self.hw_privkey, &self.pin_salt, &pin, &challenge)?;

        let cert = self.account_server.register(registration_message)?;

        let cert_claims = cert.parse_and_verify(EcdsaDecodingKey::from_pkix(&self.account_server_pubkey)?)?;
        if cert_claims.hw_pubkey.0 != *self.hw_privkey.verifying_key() {
            return Err(anyhow!("hardware pubkey did not match"));
        }

        self.registration_cert = Some(cert);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::SigningKey;
    use rand::rngs::OsRng;

    use crate::{jwt::EcdsaDecodingKey, wp::RemoteAccountServer};

    use super::Wallet;

    #[test]
    fn it_works() {
        // let (account_server, account_server_pubkey) = crate::wp::tests::new_account_server();

        // let pubkey = "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEUWwta1ybkzhRlnkVzIwDm/90alpzi6uEPXKu4vsiyOFiYNz1Ei1GVL0mNMKVUYxAjuFlYlxOf6JGkiC95RSQrA==".as_bytes().to_vec();
        // let url = "https://SSSS".to_owned();

        let pubkey = "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEouA9ULF8VKuBdHQZoaIBMIFKjo+kOAu6nDDWc9b9gw8Hf4USfFNXZUgJi37KZA6ZCTng/GBBGMzgc2T+OxXjnw==".as_bytes().to_vec();
        let url = "http://localhost:9000".to_owned();

        use base64::{engine::general_purpose::STANDARD, Engine};
        let pubkey = STANDARD.decode(pubkey).unwrap();

        let mut wallet = Wallet::new(
            RemoteAccountServer::new(url),
            pubkey.clone(),
            SigningKey::random(&mut OsRng),
        );

        wallet.register("123456".to_owned()).unwrap();

        assert!(wallet.registration_cert.is_some());
        dbg!(&wallet.registration_cert.as_ref().unwrap().0);
        dbg!(wallet
            .registration_cert
            .as_ref()
            .unwrap()
            .parse_and_verify(EcdsaDecodingKey::from_pkix(&pubkey).unwrap())
            .unwrap());
    }
}
