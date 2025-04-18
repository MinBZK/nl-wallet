/// The iterator contained multiple different items.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[error("iterator contained multiple different items")]
pub struct NotUnique;

/// Extension trait for iterators that adds a `single_unique` method.
pub trait SingleUniqueExt<T>: Iterator<Item = T> {
    /// Consumes the iterator and tries to reduce to a single unique item, returns an error if the iterator contains
    /// multiple different items (`NotUnique`)
    ///
    /// # Examples
    ///
    /// ```
    /// use utils::single_unique::{SingleUniqueExt, NotUnique};
    ///
    /// // Single item
    /// let result = vec![42].into_iter().single_unique();
    /// assert_eq!(result, Ok(Some(42)));
    ///
    /// // Empty iterator
    /// let result: Result<Option<i32>, _> = Vec::<i32>::new().into_iter().single_unique();
    /// assert_eq!(result, Ok(None));
    ///
    /// // Multiple identical items
    /// let result = vec![42, 42, 42].into_iter().single_unique();
    /// assert_eq!(result, Ok(Some(42)));
    ///
    /// // Multiple different items
    /// let result = vec![1, 2, 3].into_iter().single_unique();
    /// assert_eq!(result, Err(NotUnique));
    /// ```
    fn single_unique(self) -> Result<Option<T>, NotUnique>;
}

impl<I, T> SingleUniqueExt<T> for I
where
    I: Iterator<Item = T>,
    T: PartialEq,
{
    fn single_unique(mut self) -> Result<Option<T>, NotUnique> {
        // Try to get the first item, return None if there isn't one
        let first = match self.next() {
            Some(first) => first,
            None => return Ok(None),
        };

        // Check that all remaining items are equal to the first one
        for item in self {
            if item != first {
                return Err(NotUnique);
            }
        }

        // All items were equal to the first one (or there was only one item)
        Ok(Some(first))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_unique_empty() {
        let result: Result<Option<i32>, _> = Vec::<i32>::new().into_iter().single_unique();
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn test_single_unique_one_item() {
        let result = vec![42].into_iter().single_unique();
        assert_eq!(result, Ok(Some(42)));
    }

    #[test]
    fn test_single_unique_multiple_same_items() {
        let result = vec![42, 42, 42].into_iter().single_unique();
        assert_eq!(result, Ok(Some(42)));
    }

    #[test]
    fn test_single_unique_multiple_different_items() {
        let result = vec![1, 2, 3].into_iter().single_unique();
        assert_eq!(result, Err(NotUnique));
    }

    #[test]
    fn test_single_unique_with_custom_type() {
        #[derive(Debug, PartialEq)]
        struct TestStruct {
            value: i32,
        }

        let result = vec![TestStruct { value: 42 }, TestStruct { value: 42 }]
            .into_iter()
            .single_unique();

        assert_eq!(result, Ok(Some(TestStruct { value: 42 })));

        let result = vec![TestStruct { value: 1 }, TestStruct { value: 2 }]
            .into_iter()
            .single_unique();

        assert_eq!(result, Err(NotUnique));
    }
}
