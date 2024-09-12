use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::value::RawValue;

/// Wraps a [`RawValue`], which internally holds a string slice. Next to this, the type it serializes from and
/// deserializes to is held using [`PhantomData`]. It is used to keep track of the JSON serialization of a data
/// structure, which is necessary when signing JSON since JSON has no stable map order and can include arbitrary
/// whitespace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TypedRawValue<T>(Box<RawValue>, PhantomData<T>);

impl<T> AsRef<[u8]> for TypedRawValue<T> {
    fn as_ref(&self) -> &[u8] {
        self.0.get().as_bytes()
    }
}

impl<T> TypedRawValue<T> {
    pub fn try_new(value: &T) -> Result<Self, serde_json::Error>
    where
        T: Serialize,
    {
        let json = serde_json::to_string(value)?;
        let raw_value = RawValue::from_string(json)?;

        Ok(Self(raw_value, PhantomData))
    }

    pub fn parse(&self) -> Result<T, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        serde_json::from_str(self.0.get())
    }
}
