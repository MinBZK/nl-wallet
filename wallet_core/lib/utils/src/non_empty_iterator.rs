use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::BuildHasher;
use std::hash::Hash;

use crate::vec_non_empty;

pub trait NonEmptyIterator: IntoIterator {
    #[must_use]
    fn next(self) -> (Self::Item, Self::IntoIter)
    where
        Self: Sized,
    {
        let mut iter = self.into_iter();
        (iter.next().unwrap(), iter)
    }

    #[must_use]
    fn collect<B>(self) -> B
    where
        Self: Sized,
        B: FromNonEmptyIterator<Self::Item>,
    {
        FromNonEmptyIterator::from_non_empty_iter(self)
    }

    #[must_use]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.into_iter().fold(init, f)
    }

    #[inline]
    fn map<U, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> U,
    {
        Map {
            iter: self.into_iter().map(f),
        }
    }

    #[must_use]
    fn all<F>(self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        self.into_iter().all(f)
    }

    #[must_use]
    fn any<F>(self, f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        self.into_iter().any(f)
    }

    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        Enumerate { iter: self }
    }

    fn zip<U>(self, other: U) -> Zip<Self::IntoIter, U::IntoIter>
    where
        Self: Sized,
        U: IntoIterator,
    {
        Zip {
            inner: self.into_iter().zip(other),
        }
    }
}

/// Conversion from a [`NonEmptyIterator`].
pub trait FromNonEmptyIterator<T>: Sized {
    /// Creates a value from a [`NonEmptyIterator`].
    fn from_non_empty_iter<I>(iter: I) -> Self
    where
        I: IntoNonEmptyIterator<Item = T>;
}

impl<T> FromNonEmptyIterator<T> for Vec<T> {
    fn from_non_empty_iter<I>(iter: I) -> Self
    where
        I: IntoNonEmptyIterator<Item = T>,
    {
        iter.into_iter().collect()
    }
}

impl<T, S> FromNonEmptyIterator<T> for HashSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher + Default,
{
    fn from_non_empty_iter<I>(iter: I) -> Self
    where
        I: IntoNonEmptyIterator<Item = T>,
    {
        iter.into_iter().collect()
    }
}

impl<K, V, S> FromNonEmptyIterator<(K, V)> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    fn from_non_empty_iter<I>(iter: I) -> Self
    where
        I: IntoNonEmptyIterator<Item = (K, V)>,
    {
        iter.into_iter().collect()
    }
}

impl<A, E, V> FromNonEmptyIterator<Result<A, E>> for Result<V, E>
where
    V: FromNonEmptyIterator<A>,
{
    fn from_non_empty_iter<I>(iter: I) -> Result<V, E>
    where
        I: IntoNonEmptyIterator<Item = Result<A, E>>,
    {
        let (head, rest) = iter.into_non_empty_iter().next();
        let head: A = head?;

        let mut buf = vec_non_empty![head];

        for item in rest {
            let item: A = item?;
            buf.push(item);
        }
        let new_iter = buf.into_non_empty_iter();
        let output: V = FromNonEmptyIterator::from_non_empty_iter(new_iter);
        Ok(output)
    }
}

/// Conversion into a [`NonEmptyIterator`].
pub trait IntoNonEmptyIterator: IntoIterator {
    /// Which kind of [`NonEmptyIterator`] are we turning this into?
    type IntoNonEmptyIter: NonEmptyIterator<Item = Self::Item>;

    /// Creates a [`NonEmptyIterator`] from a value.
    fn into_non_empty_iter(self) -> Self::IntoNonEmptyIter;
}

impl<I: NonEmptyIterator> IntoNonEmptyIterator for I {
    type IntoNonEmptyIter = I;

    fn into_non_empty_iter(self) -> Self::IntoNonEmptyIter {
        self
    }
}

/// Similar to [`std::iter::Map`], but with additional non-emptiness guarantees.
#[derive(Clone)]
#[must_use = "non-empty iterators are lazy and do nothing unless consumed"]
pub struct Map<I: NonEmptyIterator, F> {
    iter: std::iter::Map<I::IntoIter, F>,
}

impl<U, I, F> NonEmptyIterator for Map<I, F>
where
    I: NonEmptyIterator,
    F: FnMut(I::Item) -> U,
{
}

/// ```
/// use utils::non_empty_iterator::NonEmptyIterator;
/// use utils::vec_non_empty;
///
/// let v: Vec<_> = vec_non_empty![1, 2, 3].non_empty_iter().map(|n| n * 2).collect();
/// ```
impl<U, I, F> IntoIterator for Map<I, F>
where
    I: NonEmptyIterator,
    F: FnMut(I::Item) -> U,
{
    type Item = U;

    type IntoIter = std::iter::Map<I::IntoIter, F>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter
    }
}

impl<I, F> std::fmt::Debug for Map<I, F>
where
    I: NonEmptyIterator,
    I::IntoIter: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.iter.fmt(f)
    }
}

#[derive(Clone)]
#[must_use = "non-empty iterators are lazy and do nothing unless consumed"]
pub struct Enumerate<I> {
    iter: I,
}

impl<I> NonEmptyIterator for Enumerate<I> where I: NonEmptyIterator {}

impl<I> IntoIterator for Enumerate<I>
where
    I: IntoIterator,
{
    type Item = (usize, I::Item);

    type IntoIter = std::iter::Enumerate<I::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter.into_iter().enumerate()
    }
}

impl<I: std::fmt::Debug> std::fmt::Debug for Enumerate<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.iter.fmt(f)
    }
}

#[derive(Clone)]
#[must_use = "non-empty iterators are lazy and do nothing unless consumed"]
pub struct Zip<A, B> {
    inner: std::iter::Zip<A, B>,
}

impl<A, B> NonEmptyIterator for Zip<A, B>
where
    A: Iterator,
    B: Iterator,
{
}

impl<A, B> IntoIterator for Zip<A, B>
where
    A: Iterator,
    B: Iterator,
{
    type Item = (A::Item, B::Item);

    type IntoIter = std::iter::Zip<A, B>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner
    }
}

impl<A, B> std::fmt::Debug for Zip<A, B>
where
    A: std::fmt::Debug,
    B: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
