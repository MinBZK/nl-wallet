pub mod pin;
pub mod pin_key;
pub mod signed;

use crate::wp::{instructions, AccountServer, WalletCertificate};

use anyhow::Result;
use p256::ecdsa::{signature::Signer, Signature, VerifyingKey};

use self::pin_key::new_pin_salt;

/// Handle to a hardware-bound ECDSA private key.
pub trait HWBoundSigningKey: Signer<Signature> {
    fn verifying_key(&self) -> VerifyingKey;
}

pub struct Wallet<T>
where
    T: HWBoundSigningKey,
{
    account_server: AccountServer,
    account_server_pubkey: Vec<u8>,
    registration_cert: Option<WalletCertificate>,

    pin_salt: Vec<u8>,
    hw_privkey: T,
}

impl<T> Wallet<T>
where
    T: HWBoundSigningKey,
{
    pub fn new(
        account_server: AccountServer,
        account_server_pubkey: Vec<u8>,
        hw_privkey: T,
    ) -> Wallet<T> {
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

        let registration_message = instructions::Registration::new_signed(
            &self.hw_privkey,
            &self.pin_salt,
            &pin,
            &challenge,
        )?;
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
