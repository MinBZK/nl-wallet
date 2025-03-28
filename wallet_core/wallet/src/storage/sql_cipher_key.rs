use std::array::TryFromSliceError;
use std::fmt::Write;

use derive_more::Constructor;

// Utility function for converting bytes to uppercase hex.
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut result, b| {
            let _ = write!(result, "{:02X}", b);
            result
        })
}

const KEY_LENGTH: usize = 32;
const SALT_LENGTH: usize = 16;

/// This represents a 32-bytes encryption key and 16-byte salt. See:
/// https://www.zetetic.net/sqlcipher/sqlcipher-api/#example-3-raw-key-data-with-explicit-salt-without-key-derivation
#[derive(Clone, Copy, Constructor)]
pub struct SqlCipherKey {
    key: [u8; KEY_LENGTH],
    salt: Option<[u8; SALT_LENGTH]>,
}

impl SqlCipherKey {
    pub fn size() -> usize {
        KEY_LENGTH
    }

    pub fn size_with_salt() -> usize {
        KEY_LENGTH + SALT_LENGTH
    }
}

/// Conversion from bytes by implementing TryFrom, which accepts a byte slice
/// of either N (no salt) or N + M (with salt) length.
impl TryFrom<&[u8]> for SqlCipherKey {
    type Error = TryFromSliceError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let key = value.get(..KEY_LENGTH).unwrap_or_default().try_into()?;
        let salt = value
            .get(KEY_LENGTH..)
            .filter(|b| !b.is_empty())
            .map(|b| b.try_into())
            .transpose()?;

        Ok(Self::new(key, salt))
    }
}

/// Conversion to a string usable in SQL statement, with or without the salt.
/// The resulting format is: x'1234ABCD'
impl From<&SqlCipherKey> for String {
    fn from(value: &SqlCipherKey) -> Self {
        let key_hex = bytes_to_hex(&value.key);
        let salt_hex = value.salt.as_ref().map(|s| bytes_to_hex(s)).unwrap_or_default();

        format!("x'{}{}'", key_hex, salt_hex)
    }
}

impl From<SqlCipherKey> for String {
    fn from(value: SqlCipherKey) -> Self {
        Self::from(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_cipher_key() {
        let key_data: [u8; 32] = [
            190, 239, 186, 190, 190, 239, 186, 190, 190, 239, 186, 190, 1, 1, 1, 1, 190, 239, 186, 190, 190, 239, 186,
            190, 190, 239, 186, 190, 190, 239, 186, 190,
        ];
        let salt_data: [u8; 16] = [202, 254, 186, 190, 1, 1, 1, 1, 202, 254, 186, 190, 202, 254, 186, 190];

        type TestSqlCipherKey = SqlCipherKey;

        assert_eq!(TestSqlCipherKey::size(), 32);
        assert_eq!(TestSqlCipherKey::size_with_salt(), 48);

        assert!(TestSqlCipherKey::try_from([].as_slice()).is_err());
        assert!(TestSqlCipherKey::try_from(&key_data[..16]).is_err());
        assert!(TestSqlCipherKey::try_from([key_data.as_slice(), &salt_data[..8]].concat().as_slice()).is_err());

        let key = TestSqlCipherKey::try_from(key_data.as_slice()).unwrap();
        assert_eq!(key.key, key_data);
        assert_eq!(key.salt, None);
        assert_eq!(
            String::from(key),
            "x'BEEFBABEBEEFBABEBEEFBABE01010101BEEFBABEBEEFBABEBEEFBABEBEEFBABE'"
        );

        let key_with_salt =
            TestSqlCipherKey::try_from([key_data.as_slice(), salt_data.as_slice()].concat().as_slice()).unwrap();
        assert_eq!(key_with_salt.key, key_data);
        assert_eq!(key_with_salt.salt, Some(salt_data));
        assert_eq!(
            String::from(key_with_salt),
            "x'BEEFBABEBEEFBABEBEEFBABE01010101BEEFBABEBEEFBABEBEEFBABEBEEFBABECAFEBABE01010101CAFEBABECAFEBABE'"
        );
    }
}
