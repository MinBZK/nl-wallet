pub mod pin;
pub mod pin_key;
pub mod signed;
pub mod signing_key;

use crate::wp::{instructions, AccountServerClient, WalletCertificate};

use anyhow::Result;
use platform_support::hw_keystore::PlatformSigningKey;

use self::pin_key::{new_pin_salt, PinKey};

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
    S: PlatformSigningKey,
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
        let pin_key = PinKey::new(&pin, &self.pin_salt);

        let registration_message =
            instructions::Registration::new_signed(&self.hw_privkey, &pin_key, &challenge)?;
        let cert = self.account_server.register(registration_message)?;

        self.registration_cert = Some(cert);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::SigningKey;
    use rand::rngs::OsRng;

    use super::Wallet;

    #[test]
    fn it_works() {
        let (account_server, account_server_pubkey) = crate::wp::tests::new_account_server();
        let mut wallet = Wallet::new(
            account_server,
            account_server_pubkey,
            SigningKey::random(&mut OsRng),
        );

        wallet.register("123456".to_owned()).unwrap();

        assert!(wallet.registration_cert.is_some());
        dbg!(wallet.registration_cert.unwrap().0);
    }
}
