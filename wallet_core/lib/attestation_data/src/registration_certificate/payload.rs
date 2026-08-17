use chrono::DateTime;
use chrono::Utc;
use dcql::ClaimsQuery;
use dcql::CredentialQueryFormat;
use serde::Deserialize;
use url::Url;
use utils::vec_at_least::VecNonEmpty;

use super::status::RegistrationCertificateStatus;

#[derive(Deserialize)]
pub struct MultiLanguageString {
    pub lang: String,
    pub value: String,
}

pub type MultiLanguageStringSet = VecNonEmpty<MultiLanguageString>;

#[derive(Deserialize)]
pub struct Credential {
    #[serde(flatten)]
    pub format: CredentialQueryFormat,
    pub claim: Option<Vec<ClaimsQuery>>,
}

#[derive(Deserialize)]
pub struct SupervisoryAuthority {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub uri: Option<Url>,
}

/// Intermediary data is parsed so it is not discarded, but its WRPAC binding is deliberately left to PVW-6062.
#[derive(Deserialize)]
pub struct Intermediary {
    pub sub: String,
    #[serde(alias = "name")]
    pub sname: Option<String>,
}

#[derive(Deserialize)]
pub struct UncheckedRegistrationCertificate {
    /// Optional at parse time because the informative Annex C example omits it. Validation always requires it.
    pub id: Option<String>,
    pub name: Option<String>,
    pub sub: String,
    pub sub_ln: Option<String>,
    pub sub_gn: Option<String>,
    pub sub_fn: Option<String>,
    pub country: String,
    pub registry_uri: Url,
    pub support_uri: String,
    pub info_uri: Option<Url>,
    pub privacy_policy: Option<Url>,
    pub srv_description: VecNonEmpty<MultiLanguageStringSet>,
    pub supervisory_authority: SupervisoryAuthority,
    pub entitlements: VecNonEmpty<Url>,
    pub credentials: Option<Vec<Credential>>,
    pub purpose: Option<MultiLanguageStringSet>,
    pub intended_use_id: Option<String>,
    pub provides_attestations: Option<Vec<Credential>>,
    pub public_body: Option<bool>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub iat: DateTime<Utc>,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub exp: Option<DateTime<Utc>>,
    pub status: RegistrationCertificateStatus,
    pub policy_id: Vec<String>,
    pub certificate_policy: Url,
    pub intermediary: Option<Intermediary>,
}

impl jwt::JwtTyp for UncheckedRegistrationCertificate {
    const TYP: &'static str = jwt::jades_b_b::JADES_B_B_JWT_TYP;
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use dcql::CredentialQueryFormat;
    use rstest::rstest;
    use serde_json::Value;
    use serde_json::json;

    use super::super::RegistrationCertificateValidationError;
    use super::super::test::ANNEX_C_EXAMPLE;
    use super::super::test::legal_person_access_certificate_subject;
    use super::super::test::valid_payload_json;
    use super::super::test::validation_time;
    use super::Credential;
    use super::UncheckedRegistrationCertificate;

    #[test]
    fn parse_annex_c_example_and_reject_its_missing_normative_id() {
        let json: Value = serde_json::from_str(ANNEX_C_EXAMPLE).unwrap();
        assert_eq!(
            json["credentials"][0]["claim"][0]["path"],
            json!(["age_equal_or_over", "18"])
        );
        let payload: UncheckedRegistrationCertificate = serde_json::from_str(ANNEX_C_EXAMPLE).unwrap();

        assert_eq!(payload.srv_description.as_slice().len(), 1);
        assert_matches!(
            &payload.credentials.as_ref().unwrap()[0].format,
            CredentialQueryFormat::SdJwt { .. }
        );
        assert_eq!(
            payload.intermediary.as_ref().unwrap().sname.as_deref(),
            Some("Intermediary Services Ltd.")
        );
        assert_matches!(
            payload.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
            Err(RegistrationCertificateValidationError::MissingId)
        );

        let mut payload_with_id: UncheckedRegistrationCertificate = serde_json::from_str(ANNEX_C_EXAMPLE).unwrap();
        payload_with_id.id = Some("wrprc-example-1".to_string());
        assert_matches!(
            payload_with_id.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
            Err(RegistrationCertificateValidationError::InvalidStatusWireFormat)
        );
    }

    #[test]
    fn reject_flat_service_description() {
        let mut json = valid_payload_json();
        json["srv_description"] = json!([{"lang": "en-US", "value": "Service"}]);

        assert!(serde_json::from_value::<UncheckedRegistrationCertificate>(json).is_err());
    }

    #[test]
    fn reject_registry_content_field_in_wrprc_payload() {
        let mut json = valid_payload_json();
        json["srv_description"] = json!([[{"lang": "en-US", "content": "Service"}]]);

        assert!(serde_json::from_value::<UncheckedRegistrationCertificate>(json).is_err());
    }

    #[test]
    fn credential_claim_values_reject_other_json_types() {
        let valid: Credential = serde_json::from_value(json!({
            "format": "dc+sd-jwt",
            "meta": { "vct_values": ["urn:eudi:pid:1"] },
            "claim": [{ "path": ["age"], "values": ["18", 18, true] }]
        }))
        .unwrap();
        assert_eq!(valid.claim.unwrap()[0].values.len(), 3);

        for invalid_value in [json!(null), json!(1.5), json!([]), json!({})] {
            let result = serde_json::from_value::<Credential>(json!({
                "format": "dc+sd-jwt",
                "meta": { "vct_values": ["urn:eudi:pid:1"] },
                "claim": [{ "path": ["age"], "values": [invalid_value] }]
            }));
            assert!(result.is_err(), "value should be rejected");
        }
    }

    #[rstest]
    #[case("registry_uri")]
    #[case("info_uri")]
    #[case("privacy_policy")]
    #[case("certificate_policy")]
    fn reject_malformed_top_level_url(#[case] field: &str) {
        let mut json = valid_payload_json();
        json[field] = json!("not a URL");

        assert!(serde_json::from_value::<UncheckedRegistrationCertificate>(json).is_err());
    }

    #[test]
    fn reject_malformed_nested_urls() {
        let mut invalid_authority = valid_payload_json();
        invalid_authority["supervisory_authority"]["uri"] = json!("not a URL");
        assert!(serde_json::from_value::<UncheckedRegistrationCertificate>(invalid_authority).is_err());

        let mut invalid_status = valid_payload_json();
        invalid_status["status"]["uri"] = json!("not a URL");
        assert!(serde_json::from_value::<UncheckedRegistrationCertificate>(invalid_status).is_err());

        let mut invalid_annex_c_status: Value = serde_json::from_str(ANNEX_C_EXAMPLE).unwrap();
        invalid_annex_c_status["status"]["status_list"]["uri"] = json!("not a URL");
        assert!(serde_json::from_value::<UncheckedRegistrationCertificate>(invalid_annex_c_status).is_err());
    }

    #[test]
    fn registration_certificate_payload_types_use_jades_type() {
        assert_eq!(
            <UncheckedRegistrationCertificate as jwt::JwtTyp>::TYP,
            jwt::jades_b_b::JADES_B_B_JWT_TYP
        );
    }
}
