use anyhow::Result;
use once_cell::sync::OnceCell;
use platform_support::hw_keystore::{error::HardwareKeyStoreError, PlatformSigningKey};

use crate::{
    account::client::{instructions::Registration, AccountServerClient, WalletCertificate},
    pin::{
        key::{new_pin_salt, PinKey},
        validation::validate_pin,
    },
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
    S: PlatformSigningKey,
{
    fn hw_privkey(&self) -> std::result::Result<&S, HardwareKeyStoreError> {
        self.hw_privkey.get_or_try_init(|| {
            let signing_key = S::signing_key(WALLET_KEY_ID)?;

            Ok::<_, HardwareKeyStoreError>(signing_key)
        })
    }

    pub fn new(account_server: T, account_server_pubkey: Vec<u8>) -> Wallet<T, S> {
        Wallet {
            account_server,
            account_server_pubkey,
            registration_cert: None,
            pin_salt: new_pin_salt(), // TODO look up in storage
            hw_privkey: OnceCell::new(),
        }
    }

    pub fn register(&mut self, pin: String) -> Result<()> {
        validate_pin(&pin)?;

        let hw_privkey = self.hw_privkey()?;
        let challenge = self.account_server.registration_challenge()?;
        let pin_key = PinKey::new(&pin, &self.pin_salt);

        let registration_message = Registration::new_signed(hw_privkey, &pin_key, &challenge)?;
        let cert = self.account_server.register(registration_message)?;

        self.registration_cert = Some(cert);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use p256::ecdsa::SigningKey;

    #[test]
    fn it_works() {
        let (account_server, account_server_pubkey) = crate::account::client::server::tests::new_account_server();
        let mut wallet: Wallet<_, SigningKey> = Wallet::new(account_server, account_server_pubkey);

        assert!(wallet.register("123456".to_owned()).is_err());

        wallet.register("112233".to_owned()).unwrap();

        assert!(wallet.registration_cert.is_some());
        dbg!(wallet.registration_cert.unwrap().0);
    }
}
