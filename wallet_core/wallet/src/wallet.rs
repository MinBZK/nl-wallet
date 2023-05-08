use anyhow::{anyhow, Result};

use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::account::{
    auth::{Registration, WalletCertificate},
    jwt::EcdsaDecodingKey,
};

use crate::{
    account_server::AccountServerClient,
    pin::{
        key::{new_pin_salt, PinKey},
        validation::validate_pin,
    },
};

const WALLET_KEY_ID: &str = "wallet";

pub struct Wallet<T, S> {
    account_server: T,
    account_server_pubkey: EcdsaDecodingKey,
    registration_cert: Option<WalletCertificate>,

    pin_salt: Vec<u8>,
    hw_privkey: S,
}

impl<T, S> Wallet<T, S>
where
    T: AccountServerClient,
    S: PlatformEcdsaKey,
{
    pub fn new(account_server: T, account_server_pubkey: EcdsaDecodingKey) -> Wallet<T, S> {
        Wallet {
            account_server,
            account_server_pubkey,
            registration_cert: None,
            pin_salt: new_pin_salt(), // TODO look up in storage
            hw_privkey: S::new(WALLET_KEY_ID),
        }
    }

    pub fn register(&mut self, pin: String) -> Result<()> {
        validate_pin(&pin)?;

        let challenge = self.account_server.registration_challenge()?;
        let pin_key = PinKey::new(&pin, &self.pin_salt);

        let registration_message = Registration::new_signed(&self.hw_privkey, &pin_key, &challenge)?;
        let cert = self.account_server.register(registration_message)?;

        let cert_claims = cert.parse_and_verify(&self.account_server_pubkey)?;
        if cert_claims.hw_pubkey.0 != self.hw_privkey.verifying_key()? {
            return Err(anyhow!("hardware pubkey did not match"));
        }

        self.registration_cert = Some(cert);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use platform_support::hw_keystore::software::SoftwareEcdsaKey;
    use wallet_provider::account_server::AccountServer;

    use super::*;

    #[test]
    fn it_works() {
        let account_server = AccountServer::new_stub();
        let pubkey = account_server.pubkey.clone();

        let mut wallet = Wallet::<_, SoftwareEcdsaKey>::new(account_server, pubkey.clone());

        assert!(wallet.register("123456".to_owned()).is_err());

        wallet.register("112233".to_owned()).unwrap();

        assert!(wallet.registration_cert.is_some());
        dbg!(&wallet.registration_cert.as_ref().unwrap().0);
        dbg!(wallet
            .registration_cert
            .as_ref()
            .unwrap()
            .parse_and_verify(&pubkey)
            .unwrap());
    }
}
