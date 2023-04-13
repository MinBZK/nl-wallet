use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;

use platform_support::hw_keystore::{HardwareKeyStoreError, PlatformEcdsaKey};
use wallet_shared::account::{
    instructions::Registration, jwt::EcdsaDecodingKey, AccountServerClient, WalletCertificate,
};

use crate::pin::{
    key::{new_pin_salt, PinKey},
    validation::validate_pin,
};

const WALLET_KEY_ID: &str = "wallet";

pub struct Wallet<T, S> {
    account_server: T,
    account_server_pubkey: Vec<u8>,
    registration_cert: Option<WalletCertificate>,

    pin_salt: Vec<u8>,
    hw_privkey: OnceCell<S>,
}

impl<T, S> Wallet<T, S>
where
    T: AccountServerClient,
    S: PlatformEcdsaKey,
{
    pub fn new(account_server: T, account_server_pubkey: Vec<u8>) -> Wallet<T, S> {
        Wallet {
            account_server,
            account_server_pubkey,
            registration_cert: None,
            pin_salt: new_pin_salt(), // TODO look up in storage
            hw_privkey: OnceCell::new(),
        }
    }

    fn hw_privkey(&self) -> Result<&S, HardwareKeyStoreError> {
        self.hw_privkey.get_or_try_init(|| S::signing_key(WALLET_KEY_ID))
    }

    pub fn register(&mut self, pin: String) -> Result<()> {
        validate_pin(&pin)?;

        let hw_privkey = self.hw_privkey()?;
        let challenge = self.account_server.registration_challenge()?;
        let pin_key = PinKey::new(&pin, &self.pin_salt);

        let registration_message = Registration::new_signed(hw_privkey, &pin_key, &challenge)?;
        let cert = self.account_server.register(registration_message)?;

        let cert_claims = cert.parse_and_verify(EcdsaDecodingKey::from_sec1(&self.account_server_pubkey)?)?;
        if cert_claims.hw_pubkey.0 != self.hw_privkey()?.verifying_key()? {
            return Err(anyhow!("hardware pubkey did not match"));
        }

        self.registration_cert = Some(cert);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose::STANDARD, Engine};

    use platform_support::hw_keystore::software::SoftwareEcdsaKey;

    use crate::account_server::RemoteAccountServer;

    use super::*;

    #[test]
    fn it_works() {
        let pubkey = "BAJXSbbTOGMvUAWhLaWp9acM3/xHdvfg7EEPyqxYDSV8gGcJ+/KUfyL9bSAlwklXu0TV6U5B8ngW4p19oNy5YrU="
            .as_bytes()
            .to_vec();
        let pubkey = STANDARD.decode(pubkey).unwrap();
        let url = "http://localhost:3000".to_owned();

        let mut wallet = Wallet::<_, SoftwareEcdsaKey>::new(RemoteAccountServer::new(url), pubkey.clone());

        assert!(wallet.register("123456".to_owned()).is_err());

        wallet.register("112233".to_owned()).unwrap();

        assert!(wallet.registration_cert.is_some());
        dbg!(&wallet.registration_cert.as_ref().unwrap().0);
        dbg!(wallet
            .registration_cert
            .as_ref()
            .unwrap()
            .parse_and_verify(EcdsaDecodingKey::from_sec1(&pubkey).unwrap())
            .unwrap());
    }
}
