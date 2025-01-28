use std::marker::PhantomData;

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
