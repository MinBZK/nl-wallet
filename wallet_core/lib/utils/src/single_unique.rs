#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Multiple items found")]
pub struct MultipleItemsFound;

pub trait SingleUnique<T> {
    fn single_unique(&mut self) -> Result<Option<T>, MultipleItemsFound>;
}

impl<I, T> SingleUnique<T> for I
where
    I: Iterator<Item = T>,
    T: PartialEq,
{
    fn single_unique(&mut self) -> Result<Option<T>, MultipleItemsFound> {
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
    use super::*;

    #[test]
    fn test_empty_iterator() {
        let empty: [i32; 0] = [];
        let result = empty.iter().single_unique();
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn test_single_item() {
        let items = [42];
        let result = items.iter().single_unique();
        assert_eq!(result, Ok(Some(&42)));
    }

    #[test]
    fn test_multiple_identical_items() {
        let items = [42, 42, 42];
        let result = items.iter().single_unique();
        assert_eq!(result, Ok(Some(&42)));
    }

    #[test]
    fn test_multiple_different_items() {
        let items = [42, 43, 42];
        let result = items.iter().single_unique();
        assert!(matches!(result, Err(MultipleItemsFound)));
    }

    #[test]
    fn test_different_types() {
        let items = ["hello", "hello", "hello"];
        let result = items.iter().single_unique();
        assert_eq!(result, Ok(Some(&"hello")));
    }
}
