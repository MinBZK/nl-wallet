pub(crate) const EXAMPLE_METADATA_BYTES: &[u8] = include_bytes!("../examples/example-metadata.json");
pub(crate) const PID_METADATA_BYTES: &[u8] = include_bytes!("../examples/pid-metadata.json");
pub(crate) const EUDI_PID_METADATA_BYTES: &[u8] = include_bytes!("../examples/example:pid:1.json");
pub(crate) const NL_PID_METADATA_BYTES: &[u8] = include_bytes!("../examples/example:pid:nl:1.json");
pub(crate) const EUDI_ADDRESS_METADATA_BYTES: &[u8] = include_bytes!("../examples/example:pid-address:1.json");
pub(crate) const NL_ADDRESS_METADATA_BYTES: &[u8] = include_bytes!("../examples/example:pid-address:nl:1.json");
pub(crate) const DEGREE_METADATA_BYTES: &[u8] = include_bytes!("../examples/degree-metadata.json");
pub(crate) const CREDENTIAL_PAYLOAD_SD_JWT_SPEC_METADATA_BYTES: &[u8] =
    include_bytes!("../examples/credential_payload_sd_jwt_metadata.json");

#[cfg(test)]
pub(crate) mod test {
    pub(crate) const RED_DOT_BYTES: &[u8] = include_bytes!("../examples/red-dot.png");

    pub(crate) const EXAMPLE_V2_METADATA_BYTES: &[u8] = include_bytes!("../examples/example-v2-metadata.json");
    pub(crate) const EXAMPLE_V3_METADATA_BYTES: &[u8] = include_bytes!("../examples/example-v3-metadata.json");
    // pub(crate) const SD_JWT_VC_SPEC_METADATA_BYTES: &[u8] =
    //     include_bytes!("../examples/spec/sd_jwt_vc_spec_metadata.json");
    pub(crate) const SIMPLE_EMBEDDED_METADATA_BYTES: &[u8] =
        include_bytes!("../examples/simple-embedded-metadata.json");
    pub(crate) const SIMPLE_REMOTE_METADATA_BYTES: &[u8] = include_bytes!("../examples/simple-remote-metadata.json");

    pub(crate) const VCT_EXAMPLE_CREDENTIAL: &str = "https://sd_jwt_vc_metadata.example.com/example_credential";
    pub(crate) const VCT_EXAMPLE_CREDENTIAL_V2: &str = "https://sd_jwt_vc_metadata.example.com/example_credential_v2";
    pub(crate) const VCT_EXAMPLE_CREDENTIAL_V3: &str = "https://sd_jwt_vc_metadata.example.com/example_credential_v3";
}
