use anyhow::Result;

use wallet_shared::account::{
    instructions::Registration, signed::SignedDouble, AccountServerClient, Certificate, Challenge, WalletCertificate,
};

pub struct RemoteAccountServer {
    url: String,
    client: reqwest::blocking::Client,
}

impl RemoteAccountServer {
    pub fn new(url: String) -> RemoteAccountServer {
        RemoteAccountServer {
            url,
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl AccountServerClient for RemoteAccountServer {
    fn registration_challenge(&self) -> Result<Vec<u8>> {
        let challenge = self
            .client
            .post(format!("{}/api/v1/enroll", self.url))
            .body("")
            .send()?
            .json::<Challenge>()?
            .challenge
            .0;
        Ok(challenge)
    }

    fn register(&self, registration_message: SignedDouble<Registration>) -> Result<WalletCertificate> {
        let cert = self
            .client
            .post(format!("{}/api/v1/createwallet", self.url))
            .json(&registration_message)
            .send()?
            .json::<Certificate>()?
            .certificate;
        Ok(cert)
    }
}

#[cfg(test)]
pub mod tests {
    use rand::rngs::OsRng;

    use platform_support::hw_keystore::software::SoftwareEcdsaKey;
    use wallet_provider::account_server::AccountServer;
    use wallet_shared::account::signing_key::EcdsaKey;

    use crate::pin::key::PinKey;

    use super::*;

    #[test]
    fn it_works() {
        // Setup wallet provider
        let account_server = AccountServer::new_stub();

        // Setup wallet
        let hw_privkey: SoftwareEcdsaKey = p256::ecdsa::SigningKey::random(&mut OsRng).into();
        let pin_privkey = PinKey::new("112233", b"salt");

        // Register
        let challenge = account_server.registration_challenge().unwrap();
        let registration_message = Registration::new_signed(&hw_privkey, &pin_privkey, &challenge).unwrap();
        let cert = account_server.register(registration_message).unwrap();

        // Verify the certificate
        let cert_data = cert.parse_and_verify(&account_server.pubkey).unwrap();
        dbg!(&cert, &cert_data);
        assert_eq!(cert_data.iss, account_server.name);
        assert_eq!(cert_data.hw_pubkey.0, hw_privkey.verifying_key().unwrap());
    }
}
