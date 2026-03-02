use std::fs;

use sd_jwt_vc_metadata::Integrity;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use utils::vec_nonempty;

/// This macro resolves a relative path to this crate's directory.
macro_rules! crate_local_path {
    ($relative:expr) => {{
        let path = std::env::current_dir().unwrap().join($relative);
        path.to_string_lossy().into_owned()
    }};
}

pub fn eudi_nl_pid_type_metadata_documents() -> (Integrity, TypeMetadataDocuments) {
    let pid_bytes = fs::read(crate_local_path!("eudi_pid_1.json")).unwrap();
    let nl_pid_bytes = fs::read(crate_local_path!("eudi_pid_nl_1.json")).unwrap();

    (
        Integrity::from(&nl_pid_bytes),
        TypeMetadataDocuments::new(vec_nonempty![pid_bytes.to_vec(), nl_pid_bytes.to_vec()]),
    )
}
