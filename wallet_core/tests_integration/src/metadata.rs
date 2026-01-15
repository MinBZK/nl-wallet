use sd_jwt_vc_metadata::Integrity;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use utils::vec_nonempty;

pub(crate) const EUDI_PID_METADATA_BYTES: &[u8] = include_bytes!("../eudi:pid:1.json");
pub(crate) const EUDI_NL_PID_METADATA_BYTES: &[u8] = include_bytes!("../eudi:pid:nl:1.json");

pub fn eudi_nl_pid_type_metadata_documents() -> (Integrity, TypeMetadataDocuments) {
    (
        Integrity::from(EUDI_NL_PID_METADATA_BYTES),
        TypeMetadataDocuments::new(vec_nonempty![
            EUDI_PID_METADATA_BYTES.to_vec(),
            EUDI_NL_PID_METADATA_BYTES.to_vec()
        ]),
    )
}
