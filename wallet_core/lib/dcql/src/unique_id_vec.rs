use derive_more::IntoIterator;
use itertools::Itertools;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de;

use utils::vec_at_least::IntoIter;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::Iter;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum UniqueIdVecError {
    #[error("source vec is empty")]
    Empty,
    #[error("source vec contains items with duplicate identifiers")]
    DuplicateIds,
}

pub trait MayHaveUniqueId {
    fn id(&self) -> Option<&str>;
}

#[derive(Debug, Clone, PartialEq, Eq, IntoIterator, Serialize)]
pub struct UniqueIdVec<T>(VecNonEmpty<T>);

impl<T> UniqueIdVec<T> {
    fn try_new(source: Vec<T>) -> Result<Self, UniqueIdVecError>
    where
        T: MayHaveUniqueId,
    {
        let vec_non_empty = VecNonEmpty::try_from(source).map_err(|_| UniqueIdVecError::Empty)?;

        if !vec_non_empty.iter().flat_map(MayHaveUniqueId::id).all_unique() {
            return Err(UniqueIdVecError::DuplicateIds);
        }

        Ok(Self(vec_non_empty))
    }

    pub fn into_inner(self) -> Vec<T> {
        let Self(vec_non_empty) = self;

        vec_non_empty.into_inner()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        let Self(vec_non_empty) = self;

        vec_non_empty.iter()
    }

    pub fn nonempty_iter(&self) -> Iter<'_, T> {
        let Self(vec_non_empty) = self;

        vec_non_empty.nonempty_iter()
    }

    pub fn into_nonempty_iter(self) -> IntoIter<T> {
        let Self(vec_non_empty) = self;

        vec_non_empty.into_nonempty_iter()
    }
}

impl<T> AsRef<[T]> for UniqueIdVec<T> {
    fn as_ref(&self) -> &[T] {
        let Self(vec_non_empty) = self;

        vec_non_empty.as_ref()
    }
}

impl<T> TryFrom<Vec<T>> for UniqueIdVec<T>
where
    T: MayHaveUniqueId,
{
    type Error = UniqueIdVecError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl<'de, T> Deserialize<'de> for UniqueIdVec<T>
where
    T: Deserialize<'de> + MayHaveUniqueId,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = Vec::<T>::deserialize(deserializer)?;
        let unique_id_vec = Self::try_new(inner).map_err(de::Error::custom)?;

        Ok(unique_id_vec)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rstest::rstest;

    use super::MayHaveUniqueId;
    use super::UniqueIdVec;
    use super::UniqueIdVecError;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct MockIdentifier<'a>(Option<&'a str>);

    impl<'a> MayHaveUniqueId for MockIdentifier<'a> {
        fn id(&self) -> Option<&str> {
            let Self(inner) = self;

            *inner
        }
    }

    #[rstest]
    #[case(vec![None], None)]
    #[case(vec![None, None], None)]
    #[case(vec![Some("foo")], None)]
    #[case(vec![Some("foo"), Some("bar")], None)]
    #[case(vec![Some("foo"), None], None)]
    #[case(vec![Some("foo"), None, Some("bar"), None], None)]
    #[case(vec![], Some(UniqueIdVecError::Empty))]
    #[case(vec![Some("foo"), Some("foo")], Some(UniqueIdVecError::DuplicateIds))]
    #[case(vec![Some("bar"), None, Some("bar"), None], Some(UniqueIdVecError::DuplicateIds))]
    fn test_unique_id_vec(#[case] source: Vec<Option<&str>>, #[case] expected_error: Option<UniqueIdVecError>) {
        let result = UniqueIdVec::try_from(source.into_iter().map(MockIdentifier).collect_vec());

        match expected_error {
            None => {
                let _ = result.expect("conversion should succeed");
            }
            Some(expected_error) => {
                assert_eq!(result.expect_err("conversion should not succeed"), expected_error);
            }
        }
    }
}
