use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::num::NonZeroUsize;

use derive_more::Index;
use itertools::Itertools;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de;
use serde_with::DeserializeAs;
use serde_with::SerializeAs;

use crate::non_empty_iterator::FromNonEmptyIterator;
use crate::non_empty_iterator::IntoNonEmptyIterator;
use crate::non_empty_iterator::NonEmptyIterator;

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

/// A macro that creates a `VecNonEmpty` from a list of expressions.
///
/// # Examples
/// ```
/// use utils::vec_non_empty;
///
/// let vec = vec_non_empty![1, 2, 3]; // Creates a VecNonEmpty<i32>
/// ```
#[macro_export]
macro_rules! vec_non_empty {
    // Version with explicit type parameter
    ($t:ty; $($x:expr),+ $(,)?) => {{
        let vec: Vec<$t> = vec![$($x),+];
        <Vec<$t> as TryInto<$crate::vec_at_least::VecNonEmpty<$t>>>::try_into(vec).unwrap()
    }};

    // Version without type parameter (relies on type inference)
    ($($x:expr),+ $(,)?) => (
        $crate::vec_at_least::VecNonEmpty::try_from(vec![$($x),+]).unwrap()
    );
}

// These should cover the most common use cases of `VecAtLeastN`.
pub type VecNonEmpty<T> = VecAtLeastN<T, 1, false>;
pub type VecNonEmptyUnique<T> = VecAtLeastN<T, 1, true>;
pub type VecAtLeastTwo<T> = VecAtLeastN<T, 2, false>;
pub type VecAtLeastTwoUnique<T> = VecAtLeastN<T, 2, true>;

/// Newtype for a [`Vec<T>`] that contains at least `N` values, with optional uniqueness validation.
/// For convenience, a number of common use cases have been defined as type aliases. Note that a
/// type with an `N` value of 0 is not valid and will cause a runtime panic when constructed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Index, Serialize, Deserialize)]
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

    pub fn last(&self) -> &T {
        // Guaranteed to succeed, as N is at least 1.
        self.0.last().unwrap()
    }

    pub fn into_last(mut self) -> T {
        // Guaranteed to succeed, as N is at least 1.
        self.0.pop().unwrap()
    }

    pub fn into_inner_last(mut self) -> (Vec<T>, T) {
        let last = self.0.pop().unwrap();

        (self.0, last)
    }

    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.0.iter_mut()
    }
}

impl<T, const N: usize> VecAtLeastN<T, N, false> {
    pub fn insert(&mut self, index: usize, element: T) {
        self.0.insert(index, element);
    }

    pub fn push(&mut self, e: T) {
        self.0.push(e);
    }

    pub fn non_empty_iter(&self) -> Iter<'_, T> {
        Iter { iter: self.0.iter() }
    }

    pub fn non_empty_iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            inner: self.0.iter_mut(),
        }
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
        value.into_inner()
    }
}

/// A non-empty iterator over the values of an [`NEVec`].
#[must_use = "non-empty iterators are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct Iter<'a, T: 'a> {
    iter: std::slice::Iter<'a, T>,
}

impl<T> NonEmptyIterator for Iter<'_, T> {}

impl<'a, T> IntoIterator for Iter<'a, T> {
    type Item = &'a T;

    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter
    }
}

impl<T: Debug> Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.iter.fmt(f)
    }
}

/// A non-empty iterator over mutable values from an [`VecNonEmpty`].
#[derive(Debug)]
#[must_use = "non-empty iterators are lazy and do nothing unless consumed"]
pub struct IterMut<'a, T: 'a> {
    inner: std::slice::IterMut<'a, T>,
}

impl<T> NonEmptyIterator for IterMut<'_, T> {}

impl<'a, T> IntoIterator for IterMut<'a, T> {
    type Item = &'a mut T;

    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner
    }
}

/// An owned non-empty iterator over values from an [`VecNonEmpty`].
#[derive(Clone)]
#[must_use = "non-empty iterators are lazy and do nothing unless consumed"]
pub struct IntoIter<T> {
    inner: std::vec::IntoIter<T>,
}

impl<T> NonEmptyIterator for IntoIter<T> {}

impl<T> IntoIterator for IntoIter<T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner
    }
}

impl<T: Debug> Debug for IntoIter<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> IntoNonEmptyIterator for VecNonEmpty<T> {
    type IntoNonEmptyIter = IntoIter<T>;

    fn into_non_empty_iter(self) -> Self::IntoNonEmptyIter {
        IntoIter {
            inner: self.0.into_iter(),
        }
    }
}

impl<'a, T> IntoNonEmptyIterator for &'a VecNonEmpty<T> {
    type IntoNonEmptyIter = Iter<'a, T>;

    fn into_non_empty_iter(self) -> Self::IntoNonEmptyIter {
        self.non_empty_iter()
    }
}

impl<T> IntoIterator for VecNonEmpty<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a VecNonEmpty<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut VecNonEmpty<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<T> FromNonEmptyIterator<T> for VecNonEmpty<T> {
    fn from_non_empty_iter<I>(iter: I) -> Self
    where
        I: IntoNonEmptyIterator<Item = T>,
    {
        VecNonEmpty::new(iter.into_iter().collect::<Vec<_>>()).unwrap()
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
    use std::collections::HashSet;
    use std::panic;

    use rstest::rstest;

    use crate::non_empty_iterator::IntoNonEmptyIterator;
    use crate::non_empty_iterator::NonEmptyIterator;

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

    #[test]
    fn test_vec_non_empty_macro() {
        let uints = vec_non_empty![u32; 1, 2, 3];
        assert_eq!(1, *uints.first());

        let str_slices = vec_non_empty!["a", "b"];
        assert_eq!("a", *str_slices.first());

        #[derive(Debug, PartialEq)]
        struct Test {
            x: usize,
        }
        let tests = vec_non_empty![Test { x: 1 }];
        assert_eq!(Test { x: 1 }, *tests.first());
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

    #[test]
    fn test_vec_non_empty_index() {
        let vec = VecNonEmpty::try_from(vec![1, 2, 3]).unwrap();
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);
        let out_of_bounds = panic::catch_unwind(|| assert_eq!(vec[3], 3));
        assert!(out_of_bounds.is_err());
    }

    #[test]
    fn test_non_empty_iter() {
        let vec = vec_non_empty![1, 2, 3];
        let iter = vec.non_empty_iter();
        let (first, mut rest_iter) = iter.next();
        assert_eq!(first, &1);
        assert_eq!(rest_iter.next(), Some(&2));
        assert_eq!(rest_iter.next(), Some(&3));
        assert_eq!(rest_iter.next(), None);
    }

    #[test]
    fn test_into_non_empty_iter() {
        let vec = vec_non_empty![1, 2, 3];
        let iter = vec.into_non_empty_iter();
        let (first, mut rest_iter) = iter.next();
        assert_eq!(first, 1);
        assert_eq!(rest_iter.next(), Some(2));
        assert_eq!(rest_iter.next(), Some(3));
        assert_eq!(rest_iter.next(), None);
    }

    #[test]
    fn test_non_empty_iter_map() {
        let vec = vec_non_empty![1, 2, 3];

        let incremented_vec = vec.non_empty_iter().map(|x| x + 1).collect::<VecNonEmpty<_>>();
        assert_eq!(incremented_vec, vec_non_empty![2, 3, 4]);

        let incremented_vec = vec.non_empty_iter().map(|x| x + 1).collect::<Vec<_>>();
        assert_eq!(incremented_vec, vec![2, 3, 4]);

        let incremented_set = vec.non_empty_iter().map(|x| x + 1).collect::<HashSet<_>>();
        assert_eq!(incremented_set, HashSet::from([2, 3, 4]));

        let into_vec = vec.into_non_empty_iter().collect::<VecNonEmpty<_>>();
        assert_eq!(into_vec, vec_non_empty![1, 2, 3]);
    }

    #[test]
    fn test_non_empty_iter_fold() {
        let vec = vec_non_empty![1, 2, 3];
        assert_eq!(6, vec.non_empty_iter().fold(0, |acc, x| acc + x));
    }
}
