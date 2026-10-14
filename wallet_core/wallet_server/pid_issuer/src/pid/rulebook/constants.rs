use attestation_types::credential_format::Format;

// # 4.2 SD-JWT VC `vct` base and NL-specific
pub const PID_BASE_SD_JWT_VCT: &str = "urn:eudi:pid:1";
pub const PID_NL_SD_JWT_VCT: &str = "urn:eudi:pid:nl:1";

// 3.1.1. mdoc `doctype` and `namespace`. Note that ISO/IEC 18013-5 has no `vct`-style extension mechanism for
// NL-specific claims (yet); those are added to the same namespace.
pub const PID_MDOC_DOCTYPE: &str = "eu.europa.ec.eudi.pid.1";
pub const PID_MDOC_NAMESPACE: &str = PID_MDOC_DOCTYPE;

pub const fn pid_attestation_type(format: Format) -> &'static str {
    match format {
        Format::MsoMdoc => PID_MDOC_DOCTYPE,
        Format::SdJwt => PID_NL_SD_JWT_VCT,
    }
}

// # 2.2 Mandatory attributes specified in CIR 2024/2977

// `family_name`; combined with partner's family name, if applicable
pub const PID_FAMILY_NAME: &str = "family_name";
pub const PID_GIVEN_NAME: &str = "given_name";

pub const PID_SD_JWT_BIRTH_DATE: &str = "birthdate";
pub const PID_MDOC_BIRTH_DATE: &str = "birth_date";

// 3.1.4 `birth_place` object
pub const PID_BIRTH_PLACE_GROUP: &str = "place_of_birth";

// `nationality`; can be multiple values
pub const PID_SD_JWT_NATIONALITIES: &str = "nationalities";
pub const PID_MDOC_NATIONALITY: &str = "nationality";

// `portrait`, mandatory in the spec, but not available in the BRP
pub const PID_SD_JWT_PORTRAIT: &str = "picture";
pub const PID_MDOC_PORTRAIT: &str = "portrait";

// # 2.3 Optional attributes specified in CIR 2024/2977

pub const PID_SD_JWT_ADDRESS_GROUP: &str = "address";
pub const PID_MDOC_RESIDENT_ADDRESS: &str = "resident_address";

// `formatted` address; unused by the PID issuer
pub const PID_SD_JWT_ADDRESS_FORMATTED: &str = "formatted";

pub const PID_SD_JWT_COUNTRY: &str = "country";
pub const PID_MDOC_RESIDENT_COUNTRY: &str = "resident_country";

pub const PID_SD_JWT_REGION: &str = "region";
pub const PID_MDOC_RESIDENT_STATE: &str = "resident_state";

pub const PID_SD_JWT_LOCALITY: &str = "locality";
pub const PID_MDOC_RESIDENT_CITY: &str = "resident_city";

pub const PID_SD_JWT_POSTAL_CODE: &str = "postal_code";
pub const PID_MDOC_RESIDENT_POSTAL_CODE: &str = "resident_postal_code";

pub const PID_SD_JWT_STREET_ADDRESS: &str = "street_address";
pub const PID_MDOC_RESIDENT_STREET: &str = "resident_street";

// For mdoc, the house number is folded into [`PID_MDOC_RESIDENT_STREET`] value.
pub const PID_SD_JWT_HOUSE_NUMBER: &str = "house_number";

// `personal_administrative_number` unused by the PID issuer
pub const PID_PERSONAL_ADMINISTRATIVE_NUMBER: &str = "personal_administrative_number";

// `family_name_birth`, can be different than `family_name`
pub const PID_SD_JWT_FAMILY_NAME_BIRTH: &str = "birth_family_name";
pub const PID_MDOC_FAMILY_NAME_BIRTH: &str = "family_name_birth";

// `given_name_birth`; not available in the BRP
pub const PID_SD_JWT_GIVEN_NAME_BIRTH: &str = "birth_given_name";
pub const PID_MDOC_GIVEN_NAME_BIRTH: &str = "given_name_birth";

pub const PID_SEX: &str = "sex";

// `email_address`; not available in the BRP
pub const PID_SD_JWT_EMAIL: &str = "email";
pub const PID_MDOC_EMAIL: &str = "email_address";

// `phone_number`; not available in the BRP
pub const PID_SD_JWT_PHONE_NUMBER: &str = "phone_number";
pub const PID_MDOC_PHONE_NUMBER: &str = "mobile_phone_number";

// # 2.4 Mandatory metadata specified in CIR 2024/2977

pub const PID_ISSUING_AUTHORITY: &str = "issuing_authority";
pub const PID_ISSUING_COUNTRY: &str = "issuing_country";

// # 2.5 Optional metadata specified in CIR 2024/2977; unused by the PID issuer

pub const PID_ISSUING_JURISDICTION: &str = "issuing_jurisdiction";

pub const PID_DOCUMENT_NUMBER: &str = "document_number";
pub const PID_SD_JWT_ISSUANCE_DATE: &str = "date_of_issuance";
pub const PID_MDOC_ISSUANCE_DATE: &str = "issuance_date";

pub const PID_SD_JWT_EXPIRY_DATE: &str = "date_of_expiry";
pub const PID_MDOC_EXPIRY_DATE: &str = "expiry_date";

// # 2.6 Additional optional attributes (not in CIR 2024/2977; added by the Rulebook itself); unused by the PID issuer

pub const PID_TRUST_ANCHOR: &str = "trust_anchor";
pub const PID_ATTESTATION_LEGAL_CATEGORY: &str = "attestation_legal_category";

// # Mandatory NL-specific attributes. Not in the Rulebook

pub const PID_BSN: &str = "bsn";
pub const PID_RECOVERY_CODE: &str = "recovery_code";
