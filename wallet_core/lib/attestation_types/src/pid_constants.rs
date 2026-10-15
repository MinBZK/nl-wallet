use crate::credential_format::Format;

pub const ROOT_PID_ATTESTATION_TYPE: &str = "urn:example:pid:1";

pub const PID_ATTESTATION_TYPE: &str = "urn:example:pid:nl:1";
pub const PID_MDOC_DOCTYPE: &str = "com.example.nl.pid.1";

/// The example PID's attestation type for `format`: its mdoc doctype or its SD-JWT `vct`.
pub const fn pid_attestation_type(format: Format) -> &'static str {
    match format {
        Format::MsoMdoc => PID_MDOC_DOCTYPE,
        Format::SdJwt => PID_ATTESTATION_TYPE,
    }
}

pub const ADDRESS_ATTESTATION_TYPE: &str = "urn:example:pid-address:nl:1";

pub const PID_FAMILY_NAME: &str = "family_name";
pub const PID_GIVEN_NAME: &str = "given_name";
pub const PID_BIRTH_DATE: &str = "birthdate";
pub const PID_AGE_OVER_18: &str = "age_over_18";
pub const PID_BSN: &str = "bsn";
pub const PID_RECOVERY_CODE: &str = "recovery_code";
pub const PID_NATIONALITY: &str = "nationalities";
pub const PID_ADDRESS_GROUP: &str = "address";
pub const PID_RESIDENT_STREET: &str = "street_address";
pub const PID_RESIDENT_HOUSE_NUMBER: &str = "house_number";
pub const PID_RESIDENT_POSTAL_CODE: &str = "postal_code";
pub const PID_RESIDENT_CITY: &str = "locality";
pub const PID_RESIDENT_COUNTRY: &str = "country";
