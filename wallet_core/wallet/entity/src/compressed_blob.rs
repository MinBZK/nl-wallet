use std::io;
use std::io::Read;
use std::io::Write;

use sea_orm::entity::prelude::*;

const DICTIONARY_VERSION: u8 = 1;
const DICTIONARY_V1: &[u8] = include_bytes!("../../attestation_copy_v1.dict");

// Custom type wrapper for compressed data
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompressedBlob(Vec<u8>);

impl CompressedBlob {
    // Compress data when creating the blob
    pub fn new(data: &[u8]) -> Result<Self, io::Error> {
        let mut encoder = zstd::Encoder::with_dictionary(Vec::with_capacity(data.len() / 2), 3, DICTIONARY_V1)?;
        encoder.write_all(data)?;
        let compressed = encoder.finish()?;

        // Prepend version byte
        let mut result = vec![DICTIONARY_VERSION];
        result.extend(compressed);
        Ok(Self(result))
    }

    // Decompress when reading
    pub fn decompress(&self) -> Result<Vec<u8>, io::Error> {
        let version = self.0[0];
        let compressed = &self.0[1..];

        let dictionary = match version {
            1 => DICTIONARY_V1,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unknown dictionary version: {}", version),
                ));
            }
        };

        let mut output = vec![];
        zstd::Decoder::with_dictionary(compressed, dictionary)?.read_to_end(&mut output)?;

        Ok(output)
    }

    // Get raw compressed bytes (for storage)
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    // Create from already-compressed bytes (when loading from DB)
    pub fn from_compressed(compressed: Vec<u8>) -> Self {
        Self(compressed)
    }
}

// Implement From traits for SeaORM conversion
impl From<CompressedBlob> for Value {
    fn from(blob: CompressedBlob) -> Self {
        Value::Bytes(Some(Box::new(blob.0)))
    }
}

impl sea_orm::TryGetable for CompressedBlob {
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let bytes: Vec<u8> = res.try_get_by(index).map_err(TryGetError::DbErr)?;
        Ok(CompressedBlob::from_compressed(bytes))
    }
}

impl sea_orm::sea_query::Nullable for CompressedBlob {
    fn null() -> Value {
        Value::Bytes(None)
    }
}

impl sea_orm::sea_query::ValueType for CompressedBlob {
    fn try_from(v: Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        match v {
            Value::Bytes(Some(bytes)) => Ok(CompressedBlob::from_compressed((*bytes).clone())),
            _ => Err(sea_orm::sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "CompressedBlob".to_string()
    }

    fn array_type() -> sea_orm::sea_query::ArrayType {
        sea_orm::sea_query::ArrayType::Bytes
    }

    fn column_type() -> sea_orm::sea_query::ColumnType {
        sea_orm::sea_query::ColumnType::VarBinary(sea_orm::sea_query::StringLen::None)
    }
}
