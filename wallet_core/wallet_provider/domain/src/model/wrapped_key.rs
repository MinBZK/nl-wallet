#[derive(Debug, Clone)]
pub struct WrappedKey(Vec<u8>);

impl WrappedKey {
    pub fn new(wrapped_key_bytes: Vec<u8>) -> Self {
        Self(wrapped_key_bytes)
    }
}

impl From<WrappedKey> for Vec<u8> {
    fn from(value: WrappedKey) -> Self {
        value.0
    }
}
