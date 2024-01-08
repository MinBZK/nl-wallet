use std::error::Error;

use crate::model::encrypted::Encrypted;

pub trait Encrypter<T> {
    type Error: Error;

    async fn encrypt(&self, key_identifier: &str, data: T) -> Result<Encrypted<T>, Self::Error>;
}

pub trait Decrypter<T> {
    type Error: Error;

    async fn decrypt(&self, key_identifier: &str, encrypted: Encrypted<T>) -> Result<T, Self::Error>;
}
