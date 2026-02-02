#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Multiple items found")]
pub struct MultipleItemsFound;

pub trait SingleUnique<T> {
    /// Reduces `T` (typically a collection or an iterator) into a single unique value.
    ///
    /// Returns
    /// - `Ok(None)` if the input is empty
    /// - `Ok(Some(value))` if all elements in `T` are equal to `value`
    /// - `Err(MultipleItemsFound)` if there are multiple different values in `T`
    fn single_unique(self) -> Result<Option<T>, MultipleItemsFound>;
}

impl<I, T> SingleUnique<T> for I
where
    I: Iterator<Item = T>,
    T: PartialEq,
{
    fn single_unique(mut self) -> Result<Option<T>, MultipleItemsFound> {
        if let Some(first) = self.next() {
            for item in self {
                if item != first {
                    return Err(MultipleItemsFound);
                }
            }
            Ok(Some(first))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(vec![], Ok(None))]
    #[case(vec![42], Ok(Some(&42)))]
    #[case(vec![42, 42, 42], Ok(Some(&42)))]
    #[case(vec![42, 43, 42], Err(MultipleItemsFound))]
    fn test_single_unique_i32(#[case] input: Vec<i32>, #[case] expected: Result<Option<&i32>, MultipleItemsFound>) {
        let result = input.iter().single_unique();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(vec!["hello", "hello", "hello"], Ok(Some(&"hello")))]
    fn test_single_unique(
        #[case] input: Vec<&'static str>,
        #[case] expected: Result<Option<&&'static str>, MultipleItemsFound>,
    ) {
        let result = input.iter().single_unique();
        assert_eq!(result, expected);
    }
}
