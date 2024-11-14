use std::num::NonZeroUsize;

use serde::de;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum NonEmptyError {
    #[error("Collection is empty")]
    Empty,
}

/// This newtype is designed to wrap any collection type and will only
/// be instantiated if the inner type has at least 1 value in it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NonEmpty<T>(T);

impl<T> NonEmpty<T> {
    /// Turn the wrapper into the wrapped type. Unfortunately implementing `Into<T>` is not possible.
    pub fn into_inner(self) -> T {
        let Self(inner) = self;

        inner
    }
}

impl<T> NonEmpty<T>
where
    for<'a> &'a T: IntoIterator,
{
    // Constructor that actually does the checking to see if the collection is not empty.
    pub fn new(collection: T) -> Result<Self, NonEmptyError> {
        // Start new context, so that the `collection_iter` is always dropped
        // and Rust does not complain about a reference held to `collection`.
        let is_empty = {
            let collection_iter = collection.into_iter();

            // Evaluate the size hint first, as this should be cheaper
            // than iterating over all the values and counting them.
            match collection_iter.size_hint() {
                (lower_bound, _) if lower_bound > 0 => false,
                (_, Some(0)) => true,
                _ => collection_iter.count() == 0,
            }
        };

        if is_empty {
            return Err(NonEmptyError::Empty);
        }

        Ok(NonEmpty(collection))
    }

    pub fn len(&self) -> NonZeroUsize {
        // We always have at least one element
        self.as_ref().into_iter().count().try_into().unwrap()
    }

    // Convenience method to return a reference to the first entry, which will always return something.
    pub fn first(&self) -> <&T as IntoIterator>::Item {
        self.as_ref().into_iter().next().unwrap()
    }
}

/// Implement [`AsRef`] for the inner type so the caller can get a reference.
impl<T> AsRef<T> for NonEmpty<T> {
    fn as_ref(&self) -> &T {
        let Self(inner) = self;

        inner
    }
}

/// Forward [`IntoIterator`] to the inner type as a handy shortcut.
impl<T> IntoIterator for NonEmpty<T>
where
    T: IntoIterator,
{
    type Item = T::Item;
    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let Self(inner) = self;

        inner.into_iter()
    }
}

// Unfortunately we cannot implement `TryFrom<T>` for `NonEmpty<T>`
// because of a blanket implementation in `std`, so we have to be
// more specific and implement it for any collection individually.
// The [`NonEmpty::new()`] method can also be called as a fallback.

impl<T> TryFrom<Vec<T>> for NonEmpty<Vec<T>> {
    type Error = NonEmptyError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// One of the main purposes of this type is to check collections
/// being deserialized from network payloads. This is why we need
/// a custom implementation of [`Deserialize`] that performs the
/// empty check.
impl<'de, T> Deserialize<'de> for NonEmpty<T>
where
    T: Deserialize<'de>,
    for<'a> &'a T: IntoIterator,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = T::deserialize(deserializer)?;

        Self::new(inner).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;
    use rstest::rstest;

    #[derive(Debug)]
    struct MockIntoIter {
        size_hint: (usize, Option<usize>),
        count: usize,
    }

    #[derive(Debug)]
    struct MockIterator {
        size_hint: (usize, Option<usize>),
        remaining_count: usize,
    }

    impl IntoIterator for &MockIntoIter {
        type Item = ();
        type IntoIter = MockIterator;

        fn into_iter(self) -> Self::IntoIter {
            MockIterator {
                size_hint: self.size_hint,
                remaining_count: self.count,
            }
        }
    }

    impl Iterator for MockIterator {
        type Item = ();

        fn next(&mut self) -> Option<Self::Item> {
            let has_next = self.remaining_count > 0;

            if has_next {
                self.remaining_count -= 1;
            }

            has_next.then_some(())
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.size_hint
        }
    }

    fn verify_non_empty_result<T: Debug, E: Debug>(actual: Result<NonEmpty<T>, E>, expect_success: bool) {
        if expect_success {
            actual.expect("should be a valid NonEmpty");
        } else {
            actual.expect_err("should not be a valid NonEmpty");
        }
    }

    #[rstest]
    #[case((0, None), 0, false)]
    #[case((0, None), 1, true)]
    #[case((0, 0.into()), 0, false)]
    #[case((1, None), 1, true)]
    fn test_non_empty_new_new(
        #[case] size_hint: (usize, Option<usize>),
        #[case] count: usize,
        #[case] expect_success: bool,
    ) {
        let collection = MockIntoIter { size_hint, count };
        let actual = NonEmpty::new(collection);
        verify_non_empty_result(actual, expect_success);
    }

    #[rstest]
    #[case(vec![()], true)]
    #[case(Vec::<()>::default(), false)]
    fn test_non_empty_from_vec(#[case] collection: Vec<()>, #[case] expect_success: bool) {
        let actual = NonEmpty::new(collection);
        verify_non_empty_result(actual, expect_success);
    }

    #[rstest]
    #[case("[0]", true)]
    #[case("[]", false)]
    fn test_non_empty_deserialize(#[case] serialized_collection: &str, #[case] expect_success: bool) {
        let actual = serde_json::from_str::<NonEmpty<Vec<u64>>>(serialized_collection);
        verify_non_empty_result(actual, expect_success);
    }

    #[test]
    fn test_non_empty_misc() {
        let non_empty = NonEmpty::try_from(vec![1, 2, 3]).unwrap();

        assert_eq!(*non_empty.first(), 1);
        assert_eq!(non_empty.len().get(), 3);
        assert_eq!(non_empty.into_inner(), [1, 2, 3]);
    }
}
