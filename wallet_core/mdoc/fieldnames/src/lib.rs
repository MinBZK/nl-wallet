/// Trait for structs that return their field names, for use in CBOR (de)serialization.
/// Can be derived, i.e., `#[derive(FieldNames)]`.
pub trait FieldNames {
    fn field_names() -> Vec<String>;
}
