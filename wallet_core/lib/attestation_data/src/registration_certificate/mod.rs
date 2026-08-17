mod payload;
mod status;
mod validation;

pub use payload::Credential;
pub use payload::Intermediary;
pub use payload::MultiLanguageString;
pub use payload::MultiLanguageStringSet;
pub use payload::SupervisoryAuthority;
pub use payload::UncheckedRegistrationCertificate;
pub use status::RegistrationCertificateStatus;
pub use status::RegistrationCertificateStatusValidationError;
pub use status::StatusValidatedRegistrationCertificate;
pub use validation::CredentialSetValidationError;
pub use validation::MultiLanguageStringSetValidationError;
pub use validation::RegistrationCertificateValidationError;
pub use validation::StructurallyValidatedRegistrationCertificate;
pub use validation::SubjectType;

#[cfg(test)]
mod test {
    use std::fmt;

    use chrono::DateTime;
    use chrono::TimeZone;
    use chrono::Utc;
    use crypto::x509::DistinguishedName;
    use serde_json::Value;
    use serde_json::json;

    use super::StatusValidatedRegistrationCertificate;
    use super::StructurallyValidatedRegistrationCertificate;
    use super::UncheckedRegistrationCertificate;
    use crate::x509::RelyingParty;

    pub(super) const ANNEX_C_EXAMPLE: &str = include_str!("../../examples/spec/registration_certificate_annex_c.json");
    pub(super) const STATUS_LIST_URI: &str = "https://example.com/statuslists/1";

    impl fmt::Debug for StructurallyValidatedRegistrationCertificate {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("StructurallyValidatedRegistrationCertificate")
        }
    }

    impl fmt::Debug for StatusValidatedRegistrationCertificate {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("StatusValidatedRegistrationCertificate")
        }
    }

    pub(super) fn valid_payload_json() -> Value {
        let mut payload: Value = serde_json::from_str(ANNEX_C_EXAMPLE).unwrap();
        payload["id"] = json!("wrprc-example-1");
        payload["status"] = json!({
            "idx": "0",
            "uri": STATUS_LIST_URI,
        });
        payload
    }

    pub(super) fn legal_person_access_certificate_subject() -> RelyingParty {
        RelyingParty::try_from(DistinguishedName::new_legal_person(
            "Example Company".to_string(),
            "DE".to_string(),
            "Example Company GmbH".to_string(),
            "LEIXG-529900T8BM49AURSDO55".to_string(),
        ))
        .unwrap()
    }

    pub(super) fn valid_payload() -> UncheckedRegistrationCertificate {
        serde_json::from_value(valid_payload_json()).unwrap()
    }

    pub(super) fn validation_time() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2023, 5, 3, 0, 0, 0).unwrap()
    }
}
