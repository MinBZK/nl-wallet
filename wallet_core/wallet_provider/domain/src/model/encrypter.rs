use std::error::Error;

use async_trait::async_trait;

use crate::model::encrypted::Encrypted;

#[async_trait]
pub trait Encrypter<T> {
    type Error: Error;

    async fn encrypt(&self, key_identifier: &str, data: T) -> Result<Encrypted<T>, Self::Error>;
}

#[async_trait]
pub trait Decrypter<T> {
    type Error: Error;

    async fn decrypt(&self, key_identifier: &str, encrypted: Encrypted<T>) -> Result<T, Self::Error>;
}
