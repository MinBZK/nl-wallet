use attestation_types::credential_format::Format;

pub const PID_BASE_SD_JWT_VCT: &str = "urn:eudi:pid:1";

/// The SD-JWT VC `vct` for the NL PID
pub const PID_NL_SD_JWT_VCT: &str = "urn:eudi:pid:nl:1";

/// The mdoc `doctype` from the PID Rulebook. Note that ISO/IEC 18013-5 has no `vct`-style extension mechanism for
/// NL specific claims
pub const PID_MDOC_DOCTYPE: &str = "eu.europa.ec.eudi.pid.1";
/// The mdoc `namespace` from the PID Rulebook. Currently equal to [`PID_MDOC_DOCTYPE`].
pub const PID_MDOC_NAMESPACE: &str = PID_MDOC_DOCTYPE;

pub fn pid_attestation_type(format: Format) -> &'static str {
    match format {
        Format::MsoMdoc => PID_MDOC_DOCTYPE,
        Format::SdJwt => PID_NL_SD_JWT_VCT,
    }
}

pub const PID_FAMILY_NAME: &str = "family_name";
pub const PID_GIVEN_NAME: &str = "given_name";
pub const PID_AGE_OVER_18: &str = "age_over_18";
pub const PID_BSN: &str = "bsn";
pub const PID_RECOVERY_CODE: &str = "recovery_code";
pub const PID_SEX: &str = "sex";

pub const PID_SD_JWT_BIRTH_DATE: &str = "birthdate";
pub const PID_MDOC_BIRTH_DATE: &str = "birth_date";

pub const PID_SD_JWT_NATIONALITIES: &str = "nationalities";
pub const PID_MDOC_NATIONALITY: &str = "nationality";

pub const PID_SD_JWT_ADDRESS_GROUP: &str = "address";
pub const PID_SD_JWT_STREET_ADDRESS: &str = "street_address";
pub const PID_SD_JWT_HOUSE_NUMBER: &str = "house_number";
pub const PID_SD_JWT_POSTAL_CODE: &str = "postal_code";
pub const PID_SD_JWT_LOCALITY: &str = "locality";
pub const PID_SD_JWT_COUNTRY: &str = "country";

pub const PID_MDOC_RESIDENT_STREET: &str = "resident_street";
pub const PID_MDOC_RESIDENT_POSTAL_CODE: &str = "resident_postal_code";
pub const PID_MDOC_RESIDENT_CITY: &str = "resident_city";
pub const PID_MDOC_RESIDENT_COUNTRY: &str = "resident_country";

/// Same name and sub-keys (`country`/`locality`) in both the mdoc and SD-JWT encodings, unlike the `resident_*`
/// address group above.
pub const PID_PLACE_OF_BIRTH_GROUP: &str = "place_of_birth";

pub const PID_ISSUING_COUNTRY: &str = "issuing_country";
pub const PID_ISSUING_AUTHORITY: &str = "issuing_authority";
