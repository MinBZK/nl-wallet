use std::hash::Hash;
use std::num::NonZeroUsize;

use itertools::Itertools;
use serde::de;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_with::DeserializeAs;
use serde_with::SerializeAs;

#[derive(Debug, thiserror::Error)]
pub enum VecAtLeastNError {
    #[error(
        "vector does not contain least {expected} {noun}, received {received}",
        noun=if *expected == 1 { "item" } else { "items" }
    )]
    TooFewItems { received: usize, expected: usize },
    #[error("at least one item appears more than once")]
    DuplicateItems,
}

// These should cover the most common use cases of `VecAtLeastN`.
pub type VecNonEmpty<T> = VecAtLeastN<T, 1, false>;
pub type VecAtLeastTwo<T> = VecAtLeastN<T, 2, false>;
pub type VecAtLeastTwoUnique<T> = VecAtLeastN<T, 2, true>;

/// Newtype for a [`Vec<T>`] that contains at least `N` values, with optional uniquness validation.
/// For convenience, a number of common use cases have been defined as type aliases. Note that a
/// type with an `N` value of 0 is not valid and will cause a runtime panic when constructed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VecAtLeastN<T, const N: usize, const UNIQUE: bool>(Vec<T>);

impl<T, const N: usize, const UNIQUE: bool> VecAtLeastN<T, N, UNIQUE> {
    fn new(inner: Vec<T>) -> Result<Self, VecAtLeastNError> {
        // Unfortunately this cannot be a compile time check on N, so
        // it is checked at runtime in the type's only constructor.
        assert!(N > 0, "minimum length N must be a positive integer");

        // Check for at least N items.
        let length = inner.len();

        if length >= N {
            Ok(Self(inner))
        } else {
            Err(VecAtLeastNError::TooFewItems {
                received: length,
                expected: N,
            })
        }
    }

    pub fn len(&self) -> NonZeroUsize {
        // Guaranteed to succeed, as N is at least 1.
        self.0.len().try_into().unwrap()
    }

    pub fn first(&self) -> &T {
        // Guaranteed to succeed, as N is at least 1.
        self.0.first().unwrap()
    }

    pub fn into_first(self) -> T {
        // Guaranteed to succeed, as N is at least 1.
        self.0.into_iter().next().unwrap()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

/// Should be used as the constructor for types where the uniqueness constraint is set.
impl<T, const N: usize> TryFrom<Vec<T>> for VecAtLeastN<T, N, false> {
    type Error = VecAtLeastNError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Should be used as the constructor for types that require uniqeness
/// among its items.  Note that this places extra trait bounds on `T`.
impl<T, const N: usize> TryFrom<Vec<T>> for VecAtLeastN<T, N, true>
where
    T: Eq + Hash,
{
    type Error = VecAtLeastNError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        // Perform the check on the number of items in the internal constructor.
        let vec_at_least = Self::new(value)?;

        // Additionally check for uniqueness.
        if !vec_at_least.0.iter().all_unique() {
            return Err(VecAtLeastNError::DuplicateItems);
        }

        Ok(vec_at_least)
    }
}

impl<T, const N: usize, const UNIQUE: bool> AsRef<[T]> for VecAtLeastN<T, N, UNIQUE> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize, const UNIQUE: bool> From<VecAtLeastN<T, N, UNIQUE>> for Vec<T> {
    fn from(value: VecAtLeastN<T, N, UNIQUE>) -> Self {
        value.into_vec()
    }
}

// The trait implementations below allow this type to be used in combination
// with `serde_with` by specifying a `#[serde_as(as = "Vec<T>")]` macro.

impl<T, U, const N: usize, const UNIQUE: bool> SerializeAs<VecAtLeastN<T, N, UNIQUE>> for Vec<U>
where
    U: SerializeAs<T>,
{
    fn serialize_as<S>(source: &VecAtLeastN<T, N, UNIQUE>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Use the `SerializeAs<Vec<T>> for Vec<U>` trait implementation.
        Vec::<U>::serialize_as(&source.0, serializer)
    }
}

impl<'de, T, U, const N: usize> DeserializeAs<'de, VecAtLeastN<T, N, false>> for Vec<U>
where
    U: DeserializeAs<'de, T>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<VecAtLeastN<T, N, false>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Use the `DeserializeAs<'de, Vec<T>> for Vec<U>` trait implementation.
        let vec: Vec<T> = Vec::<U>::deserialize_as(deserializer)?;
        let vec_at_least = vec.try_into().map_err(de::Error::custom)?;

        Ok(vec_at_least)
    }
}

impl<'de, T, U, const N: usize> DeserializeAs<'de, VecAtLeastN<T, N, true>> for Vec<U>
where
    T: Eq + Hash,
    U: DeserializeAs<'de, T>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<VecAtLeastN<T, N, true>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Use the `DeserializeAs<'de, Vec<T>> for Vec<U>` trait implementation.
        let vec: Vec<T> = Vec::<U>::deserialize_as(deserializer)?;
        let vec_at_least = vec.try_into().map_err(de::Error::custom)?;

        Ok(vec_at_least)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::VecAtLeastN;
    use super::VecAtLeastTwo;
    use super::VecAtLeastTwoUnique;
    use super::VecNonEmpty;

    #[test]
    #[should_panic]
    fn test_vec_at_least_n0_panic() {
        let _ = VecAtLeastN::<(), 0, false>::try_from(vec![]);
    }

    #[test]
    #[should_panic]
    fn test_vec_at_least_n0_unique_panic() {
        let _ = VecAtLeastN::<(), 0, true>::try_from(vec![]);
    }

    #[rstest]
    #[case(vec![], false)]
    #[case(vec![1], true)]
    #[case(vec![1, 2], true)]
    #[case(vec![1, 2, 3], true)]
    fn test_vec_non_empty(#[case] input: Vec<usize>, #[case] expected_is_ok: bool) {
        let vec = VecNonEmpty::try_from(input);

        assert_eq!(vec.is_ok(), expected_is_ok);
    }

    #[rstest]
    #[case(vec![], false)]
    #[case(vec![1], false)]
    #[case(vec![1, 2], true)]
    #[case(vec![1, 2, 3], true)]
    #[case(vec![1, 1], true)]
    #[case(vec![1, 2, 1], true)]
    fn test_vec_at_least_two(#[case] input: Vec<usize>, #[case] expected_is_ok: bool) {
        let vec = VecAtLeastTwo::try_from(input);

        assert_eq!(vec.is_ok(), expected_is_ok);
    }

    #[rstest]
    #[case(vec![], false)]
    #[case(vec![1], false)]
    #[case(vec![1, 2], true)]
    #[case(vec![1, 2, 3], true)]
    #[case(vec![1, 1], false)]
    #[case(vec![1, 2, 1], false)]
    fn test_vec_at_least_two_unique(#[case] input: Vec<usize>, #[case] expected_is_ok: bool) {
        let vec = VecAtLeastTwoUnique::try_from(input);

        assert_eq!(vec.is_ok(), expected_is_ok);
    }
}
