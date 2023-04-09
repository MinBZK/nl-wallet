use anyhow::Result;
use once_cell::sync::OnceCell;
use platform_support::hw_keystore::{HardwareKeyStoreError, PlatformSigningKey};

use crate::pin::{
    key::{new_pin_salt, PinKey},
    validation::validate_pin,
};
use wallet_shared::account::{instructions::Registration, AccountServerClient, WalletCertificate};

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

        self.registration_cert = Some(cert);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use platform_support::hw_keystore::software::SoftwareSigningKey;

    #[test]
    fn it_works() {
        let (account_server, account_server_pubkey) = crate::account_server::tests::new_account_server();
        let mut wallet: Wallet<_, SoftwareSigningKey> = Wallet::new(account_server, account_server_pubkey);

        assert!(wallet.register("123456".to_owned()).is_err());

        wallet.register("112233".to_owned()).unwrap();

        assert!(wallet.registration_cert.is_some());
        dbg!(wallet.registration_cert.unwrap().0);
    }
}
