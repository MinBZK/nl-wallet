use std::io;
use std::io::Read;
use std::io::Write;

use sea_orm::entity::prelude::*;

const DICTIONARY: &[u8] = include_bytes!("../../attestation_copy.dict");

// Custom type wrapper for compressed data
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompressedBlob(Vec<u8>);

impl CompressedBlob {
    // Compress data when creating the blob
    pub fn new(data: &[u8]) -> Result<Self, io::Error> {
        let mut output_buffer = vec![];
        let mut encoder = zstd::Encoder::with_dictionary(&mut output_buffer, 3, DICTIONARY)?;
        encoder.write_all(data)?;
        encoder.auto_finish();

        Ok(Self(output_buffer))
    }

    // Decompress when reading
    pub fn decompress(&self) -> Result<Vec<u8>, io::Error> {
        let mut output = vec![];
        zstd::Decoder::with_dictionary(&self.0[..], DICTIONARY)?.read_to_end(&mut output)?;

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
