use std::ops::Deref;

use serde::{de, Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum NonEmptyError {
    #[error("Collection is empty")]
    Empty,
}

#[derive(Debug, Clone, Serialize)]
pub struct NonEmptyVec<T>(Vec<T>);

impl<T> NonEmptyVec<T> {
    fn new(inner: Vec<T>) -> Result<Self, NonEmptyError> {
        if inner.is_empty() {
            return Err(NonEmptyError::Empty);
        }

        Ok(NonEmptyVec(inner))
    }

    pub fn into_inner(self) -> Vec<T> {
        let NonEmptyVec(inner) = self;

        inner
    }
}

impl<T> TryFrom<Vec<T>> for NonEmptyVec<T> {
    type Error = NonEmptyError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<T> Deref for NonEmptyVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        let NonEmptyVec(inner) = self;

        inner
    }
}

impl<'de, T> Deserialize<'de> for NonEmptyVec<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = Vec::deserialize(deserializer)?;

        Self::new(inner).map_err(de::Error::custom)
    }
}
