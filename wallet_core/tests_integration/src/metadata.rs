use sd_jwt_vc_metadata::Integrity;
use sd_jwt_vc_metadata::TypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use sd_jwt_vc_metadata::UncheckedTypeMetadata;
use utils::vec_nonempty;

pub(crate) const EUDI_PID_METADATA_BYTES: &[u8] = include_bytes!("../eudi:pid:1.json");
pub(crate) const EUDI_NL_PID_METADATA_BYTES: &[u8] = include_bytes!("../eudi:pid:nl:1.json");
pub(crate) const EXAMPLE_DEGREE_METADATA_BYTES: &[u8] = include_bytes!("../com.example.degree.json");
pub(crate) const EXAMPLE_HOUSING_METADATA_BYTES: &[u8] = include_bytes!("../com.example.housing.json");
pub(crate) const EXAMPLE_INSURANCE_METADATA_BYTES: &[u8] = include_bytes!("../com.example.insurance.json");

// UncheckedTypeMetadata

pub fn eudi_pid_unchecked_type_metadata() -> UncheckedTypeMetadata {
    serde_json::from_slice(EUDI_PID_METADATA_BYTES).unwrap()
}

pub fn eudi_nl_pid_unchecked_type_metadata() -> UncheckedTypeMetadata {
    serde_json::from_slice(EUDI_NL_PID_METADATA_BYTES).unwrap()
}

pub fn example_degree_unchecked_type_metadata() -> UncheckedTypeMetadata {
    serde_json::from_slice(EXAMPLE_DEGREE_METADATA_BYTES).unwrap()
}

pub fn example_housing_unchecked_type_metadata() -> UncheckedTypeMetadata {
    serde_json::from_slice(EXAMPLE_HOUSING_METADATA_BYTES).unwrap()
}

pub fn example_insurance_unchecked_type_metadata() -> UncheckedTypeMetadata {
    serde_json::from_slice(EXAMPLE_INSURANCE_METADATA_BYTES).unwrap()
}

// TypeMetadata

pub fn eudi_pid_type_metadata() -> TypeMetadata {
    TypeMetadata::try_new(eudi_pid_unchecked_type_metadata()).unwrap()
}

pub fn eudi_nl_pid_type_metadata() -> TypeMetadata {
    TypeMetadata::try_new(eudi_nl_pid_unchecked_type_metadata()).unwrap()
}

pub fn example_degree_type_metadata() -> TypeMetadata {
    TypeMetadata::try_new(example_degree_unchecked_type_metadata()).unwrap()
}

pub fn example_housing_type_metadata() -> TypeMetadata {
    TypeMetadata::try_new(example_housing_unchecked_type_metadata()).unwrap()
}

pub fn example_insurance_type_metadata() -> TypeMetadata {
    TypeMetadata::try_new(example_insurance_unchecked_type_metadata()).unwrap()
}

// TypeMetadataDocuments

pub fn eudi_nl_pid_type_metadata_documents() -> (Integrity, TypeMetadataDocuments) {
    (
        Integrity::from(EUDI_NL_PID_METADATA_BYTES),
        TypeMetadataDocuments::new(vec_nonempty![
            EUDI_PID_METADATA_BYTES.to_vec(),
            EUDI_NL_PID_METADATA_BYTES.to_vec()
        ]),
    )
}

pub fn example_degree_type_metadata_documents() -> (Integrity, TypeMetadataDocuments) {
    (
        Integrity::from(EXAMPLE_DEGREE_METADATA_BYTES),
        TypeMetadataDocuments::new(vec_nonempty![EXAMPLE_DEGREE_METADATA_BYTES.to_vec()]),
    )
}

pub fn example_housing_type_metadata_documents() -> (Integrity, TypeMetadataDocuments) {
    (
        Integrity::from(EXAMPLE_HOUSING_METADATA_BYTES),
        TypeMetadataDocuments::new(vec_nonempty![EXAMPLE_HOUSING_METADATA_BYTES.to_vec()]),
    )
}

pub fn example_insurance_type_metadata_documents() -> (Integrity, TypeMetadataDocuments) {
    (
        Integrity::from(EXAMPLE_INSURANCE_METADATA_BYTES),
        TypeMetadataDocuments::new(vec_nonempty![EXAMPLE_INSURANCE_METADATA_BYTES.to_vec()]),
    )
}
