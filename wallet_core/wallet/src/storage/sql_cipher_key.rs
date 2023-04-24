use std::array::TryFromSliceError;

use rusqlite::{
    types::{ToSqlOutput, Value},
    ToSql,
};

// Utility function for converting bytes to uppercase hex.
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02X}", b)).collect()
}

/// Filled in const generics for struct below, only one actually supported by SQLCipher.
const AES_KEY_LENGTH: usize = 32;
const AES_SALT_LENGTH: usize = 16;

pub type Aes256SqlCipherKey = SqlCipherKey<AES_KEY_LENGTH, AES_SALT_LENGTH>;

/// This represents a 32-bytes encryption key and 16-byte salt. See:
/// https://www.zetetic.net/sqlcipher/sqlcipher-api/#example-3-raw-key-data-with-explicit-salt-without-key-derivation
#[derive(Clone, Copy)]
pub struct SqlCipherKey<const N: usize, const M: usize> {
    key: [u8; N],
    salt: Option<[u8; M]>,
}

impl<const N: usize, const M: usize> SqlCipherKey<N, M> {
    pub fn new(key: [u8; N], salt: Option<[u8; M]>) -> Self {
        SqlCipherKey { key, salt }
    }

    pub fn size() -> usize {
        N
    }

    pub fn size_with_salt() -> usize {
        N + M
    }
}

/// Conversion from bytes by implementing TryFrom, which accepts a byte slice
/// of either N (no salt) or N + M (with salt) length.
impl<const N: usize, const M: usize> TryFrom<&[u8]> for SqlCipherKey<N, M> {
    type Error = TryFromSliceError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let key = value.get(..N).unwrap_or_default().try_into()?;
        let salt = value
            .get(N..)
            .filter(|b| !b.is_empty())
            .map(|b| b.try_into())
            .transpose()?;

        Ok(Self::new(key, salt))
    }
}

/// Convertion to a string usable in SQL statement, with or without the salt.
/// The resulting format is: x'1234ABCD'
impl<const N: usize, const M: usize> ToSql for SqlCipherKey<N, M> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let key_hex = bytes_to_hex(&self.key);
        let salt_hex = self.salt.as_ref().map(|s| bytes_to_hex(s)).unwrap_or_default();

        let key = format!("x'{}{}'", key_hex, salt_hex);

        Ok(ToSqlOutput::Owned(Value::Text(key)))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_sql_cipher_key() -> Result<()> {
        type TestSqlCipherKey = SqlCipherKey<2, 3>;

        assert_eq!(TestSqlCipherKey::size(), 2);
        assert_eq!(TestSqlCipherKey::size_with_salt(), 5);

        assert!(TestSqlCipherKey::try_from([].as_slice()).is_err());
        assert!(TestSqlCipherKey::try_from([190].as_slice()).is_err());
        assert!(TestSqlCipherKey::try_from([190, 239, 186, 190].as_slice()).is_err());

        let key = TestSqlCipherKey::try_from([190, 239].as_slice())?;
        assert_eq!(key.key, [190, 239]);
        assert_eq!(key.salt, None);
        assert_eq!(key.to_sql(), Ok(ToSqlOutput::Owned(Value::Text("x'BEEF'".to_string()))));

        let key = TestSqlCipherKey::try_from([190, 239, 0, 186, 190].as_slice())?;
        assert_eq!(key.key, [190, 239]);
        assert_eq!(key.salt, Some([0, 186, 190]));
        assert_eq!(
            key.to_sql(),
            Ok(ToSqlOutput::Owned(Value::Text("x'BEEF00BABE'".to_string())))
        );

        Ok(())
    }
}
