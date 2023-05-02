use anyhow::Result;

use wallet_common::account::{
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
