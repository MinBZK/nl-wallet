use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::value::RawValue;

/// Wraps a [`RawValue`], which internally holds a string slice. Next to this, the type it serializes from and
/// deserializes to is held using [`PhantomData`]. It is used to keep track of the JSON serialization of a data
/// structure, which is necessary when signing JSON since JSON has no stable map order and can include arbitrary
/// whitespace.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct TypedRawValue<T>(Box<RawValue>, PhantomData<T>);

// Implement `Clone` manually, as there would be an unnecessary trait bound `T: Clone` if `Clone` were to be derived.
impl<T> Clone for TypedRawValue<T> {
    fn clone(&self) -> Self {
        let Self(raw_value, _) = self;

        Self(raw_value.clone(), PhantomData)
    }
}

impl<T> AsRef<[u8]> for TypedRawValue<T> {
    fn as_ref(&self) -> &[u8] {
        let Self(raw_value, _) = self;

        raw_value.get().as_bytes()
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
        let Self(raw_value, _) = self;

        serde_json::from_str(raw_value.get())
    }
}
