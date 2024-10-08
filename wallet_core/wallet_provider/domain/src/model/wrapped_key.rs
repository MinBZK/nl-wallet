use p256::ecdsa::VerifyingKey;

#[derive(Debug, Clone)]
pub struct WrappedKey {
    wrapped_private_key: Vec<u8>,
    public_key: VerifyingKey,
}

impl WrappedKey {
    pub fn new(wrapped_private_key: Vec<u8>, public_key: VerifyingKey) -> Self {
        Self {
            wrapped_private_key,
            public_key,
        }
    }

    pub fn public_key(&self) -> &VerifyingKey {
        &self.public_key
    }
}

impl From<WrappedKey> for Vec<u8> {
    fn from(value: WrappedKey) -> Self {
        value.wrapped_private_key
    }
}
