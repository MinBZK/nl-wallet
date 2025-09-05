use std::{fmt::Debug, marker::PhantomData};

#[derive(Clone)]
pub struct InitializationVector(pub Vec<u8>);

#[derive(Clone)]
pub struct Encrypted<T> {
    pub data: Vec<u8>,
    pub iv: InitializationVector,
    _decrypted_data: PhantomData<T>,
}

impl<T> Encrypted<T> {
    pub fn new(data: Vec<u8>, iv: InitializationVector) -> Self {
        Self {
            data,
            iv,
            _decrypted_data: PhantomData,
        }
    }
}

impl<T> Debug for Encrypted<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Encrypted")
            .field("iv", &self.iv.0)
            .field("_decrypted_data", &self._decrypted_data)
            .finish_non_exhaustive()
    }
}
