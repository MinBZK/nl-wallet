use dcql::ClaimsQuery;
use dcql::ClaimsSelection;
use dcql::CredentialQuery;
use dcql::CredentialQueryFormat;
use dcql::CredentialQueryIdentifier;
use dcql::Query;

use super::Credential;
use super::StatusValidatedRegistrationCertificate;

#[derive(Debug, thiserror::Error)]
pub enum RegistrationCertificateAuthorizationError {
    #[error("registration certificate does not authorize credential query `{0}`")]
    UnauthorizedCredential(CredentialQueryIdentifier),
}

impl StatusValidatedRegistrationCertificate {
    /// Validate that every credential, claim and requested value in a DCQL query is authorized by this certificate.
    pub fn validate_query_authorization(&self, query: &Query) -> Result<(), RegistrationCertificateAuthorizationError> {
        validate_query_authorization(query, self.payload().credentials.as_deref().unwrap_or_default())
    }
}

fn validate_query_authorization(
    query: &Query,
    authorized_credentials: &[Credential],
) -> Result<(), RegistrationCertificateAuthorizationError> {
    for credential_query in query.credentials.iter() {
        if !authorized_credentials
            .iter()
            .any(|authorized| credential_query_is_authorized(credential_query, authorized))
        {
            return Err(RegistrationCertificateAuthorizationError::UnauthorizedCredential(
                credential_query.id.clone(),
            ));
        }
    }

    Ok(())
}

fn credential_query_is_authorized(query: &CredentialQuery, authorized: &Credential) -> bool {
    if !format_is_authorized(&query.format, &authorized.format) {
        return false;
    }

    let requested_claims = match &query.claims_selection {
        ClaimsSelection::NoSelectivelyDisclosable => return true,
        ClaimsSelection::Combinations { claims, .. } | ClaimsSelection::All { claims } => claims,
    };

    let Some(authorized_claims) = authorized.claim.as_deref() else {
        return false;
    };

    requested_claims.iter().all(|requested| {
        authorized_claims
            .iter()
            .any(|authorized| claim_is_authorized(requested, authorized))
    })
}

fn format_is_authorized(requested: &CredentialQueryFormat, authorized: &CredentialQueryFormat) -> bool {
    match (requested, authorized) {
        (
            CredentialQueryFormat::MsoMdoc {
                doctype_value: requested,
            },
            CredentialQueryFormat::MsoMdoc {
                doctype_value: authorized,
            },
        ) => requested == authorized,
        (
            CredentialQueryFormat::SdJwt { vct_values: requested },
            CredentialQueryFormat::SdJwt { vct_values: authorized },
        ) => requested
            .iter()
            .all(|value| authorized.iter().any(|authorized| authorized == value)),
        _ => false,
    }
}

fn claim_is_authorized(requested: &ClaimsQuery, authorized: &ClaimsQuery) -> bool {
    if requested.path != authorized.path {
        return false;
    }

    authorized.values.is_empty()
        || (!requested.values.is_empty() && requested.values.iter().all(|value| authorized.values.contains(value)))
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use dcql::ClaimsSelection;
    use dcql::CredentialQuery;
    use dcql::Query;
    use rstest::rstest;
    use serde_json::Value;
    use serde_json::json;

    use super::Credential;
    use super::RegistrationCertificateAuthorizationError;
    use super::validate_query_authorization;

    fn credential(value: Value) -> Credential {
        serde_json::from_value(value).unwrap()
    }

    fn query_with_credentials(credentials: &[Value]) -> Query {
        serde_json::from_value(json!({ "credentials": credentials })).unwrap()
    }

    fn query(value: Value) -> Query {
        query_with_credentials(&[value])
    }

    fn sd_jwt_query_value(id: &str, vct_values: &Value, claims: &Value) -> Value {
        json!({
            "id": id,
            "format": "dc+sd-jwt",
            "meta": { "vct_values": vct_values },
            "claims": claims
        })
    }

    fn sd_jwt_query(vct_values: &Value, claims: &Value) -> Query {
        query(sd_jwt_query_value("pid", vct_values, claims))
    }

    fn authorized_sd_jwt() -> Credential {
        credential(json!({
            "format": "dc+sd-jwt",
            "meta": { "vct_values": ["urn:example:pid", "urn:example:pid-derived"] },
            "claim": [
                { "path": ["age"], "values": [18, 21] },
                { "path": ["family_name"] }
            ]
        }))
    }

    #[test]
    fn authorize_query_when_format_claims_and_values_are_subsets() {
        let query = sd_jwt_query(
            &json!(["urn:example:pid"]),
            &json!([
                { "path": ["age"], "values": [18] },
                { "path": ["family_name"] }
            ]),
        );

        validate_query_authorization(&query, &[authorized_sd_jwt()]).unwrap();
    }

    #[test]
    fn reject_query_when_authorized_credentials_are_empty() {
        let query = sd_jwt_query(&json!(["urn:example:pid"]), &json!([{ "path": ["family_name"] }]));

        assert_matches!(
            validate_query_authorization(&query, &[]),
            Err(RegistrationCertificateAuthorizationError::UnauthorizedCredential(_))
        );
    }

    #[test]
    fn authorize_multiple_queries_using_distinct_registration_certificate_entries() {
        let query = query_with_credentials(&[
            sd_jwt_query_value(
                "pid",
                &json!(["urn:example:pid"]),
                &json!([{ "path": ["family_name"] }]),
            ),
            sd_jwt_query_value(
                "address",
                &json!(["urn:example:address"]),
                &json!([{ "path": ["street"] }]),
            ),
        ]);
        let authorized = [
            credential(json!({
                "format": "dc+sd-jwt",
                "meta": { "vct_values": ["urn:example:address"] },
                "claim": [{ "path": ["street"] }]
            })),
            authorized_sd_jwt(),
        ];

        validate_query_authorization(&query, &authorized).unwrap();
    }

    #[test]
    fn reject_when_any_of_multiple_queries_is_unauthorized() {
        let query = query_with_credentials(&[
            sd_jwt_query_value(
                "pid",
                &json!(["urn:example:pid"]),
                &json!([{ "path": ["family_name"] }]),
            ),
            sd_jwt_query_value(
                "address",
                &json!(["urn:example:address"]),
                &json!([{ "path": ["street"] }]),
            ),
        ]);

        assert_matches!(
            validate_query_authorization(&query, &[authorized_sd_jwt()]),
            Err(RegistrationCertificateAuthorizationError::UnauthorizedCredential(id))
                if id.as_ref() == "address"
        );
    }

    #[rstest]
    #[case::unauthorized_vct(
        json!(["urn:example:pid", "urn:example:address"]),
        json!([{ "path": ["family_name"] }])
    )]
    #[case::unauthorized_claim_path(
        json!(["urn:example:pid"]),
        json!([{ "path": ["given_name"] }])
    )]
    #[case::value_outside_authorized_values(
        json!(["urn:example:pid"]),
        json!([{ "path": ["age"], "values": [18, 65] }])
    )]
    #[case::unrestricted_value_request(
        json!(["urn:example:pid"]),
        json!([{ "path": ["age"] }])
    )]
    fn reject_unauthorized_sd_jwt_query(#[case] vct_values: Value, #[case] claims: Value) {
        let query = sd_jwt_query(&vct_values, &claims);

        assert_matches!(
            validate_query_authorization(&query, &[authorized_sd_jwt()]),
            Err(RegistrationCertificateAuthorizationError::UnauthorizedCredential(_))
        );
    }

    #[test]
    fn reject_query_when_authorization_is_split_across_credentials() {
        let query = sd_jwt_query(
            &json!(["urn:example:pid", "urn:example:pid-derived"]),
            &json!([{ "path": ["family_name"] }]),
        );
        let authorized = [
            credential(json!({
                "format": "dc+sd-jwt",
                "meta": { "vct_values": ["urn:example:pid"] },
                "claim": [{ "path": ["family_name"] }]
            })),
            credential(json!({
                "format": "dc+sd-jwt",
                "meta": { "vct_values": ["urn:example:pid-derived"] },
                "claim": [{ "path": ["family_name"] }]
            })),
        ];

        assert_matches!(
            validate_query_authorization(&query, &authorized),
            Err(RegistrationCertificateAuthorizationError::UnauthorizedCredential(_))
        );
    }

    #[test]
    fn authorize_requested_values_when_registration_certificate_does_not_restrict_values() {
        let query = sd_jwt_query(
            &json!(["urn:example:pid"]),
            &json!([{ "path": ["age"], "values": [18, 21] }]),
        );
        let authorized = credential(json!({
            "format": "dc+sd-jwt",
            "meta": { "vct_values": ["urn:example:pid"] },
            "claim": [{ "path": ["age"] }]
        }));

        validate_query_authorization(&query, &[authorized]).unwrap();
    }

    #[rstest]
    #[case::missing(json!({
        "format": "dc+sd-jwt",
        "meta": { "vct_values": ["urn:example:pid"] }
    }))]
    #[case::empty(json!({
        "format": "dc+sd-jwt",
        "meta": { "vct_values": ["urn:example:pid"] },
        "claim": []
    }))]
    fn reject_claim_query_when_registration_certificate_has_no_claim_authorization(#[case] authorized: Value) {
        let query = sd_jwt_query(&json!(["urn:example:pid"]), &json!([{ "path": ["family_name"] }]));
        let authorized = credential(authorized);

        assert_matches!(
            validate_query_authorization(&query, &[authorized]),
            Err(RegistrationCertificateAuthorizationError::UnauthorizedCredential(_))
        );
    }

    #[test]
    fn authorize_query_without_selectively_disclosable_claims() {
        let mut credential_query: CredentialQuery = serde_json::from_value(sd_jwt_query_value(
            "pid",
            &json!(["urn:example:pid"]),
            &json!([{ "path": ["family_name"] }]),
        ))
        .unwrap();
        credential_query.claims_selection = ClaimsSelection::NoSelectivelyDisclosable;
        let query = Query {
            credentials: vec![credential_query].try_into().unwrap(),
            credential_sets: vec![],
        };
        let authorized = credential(json!({
            "format": "dc+sd-jwt",
            "meta": { "vct_values": ["urn:example:pid"] }
        }));

        validate_query_authorization(&query, &[authorized]).unwrap();
    }

    #[test]
    fn claim_sets_require_every_alternative_to_be_authorized() {
        let query = query(json!({
            "id": "pid",
            "format": "dc+sd-jwt",
            "meta": { "vct_values": ["urn:example:pid"] },
            "claims": [
                { "id": "name", "path": ["family_name"] },
                { "id": "age", "path": ["age"] }
            ],
            "claim_sets": [["name"], ["age"]]
        }));
        let fully_authorized = credential(json!({
            "format": "dc+sd-jwt",
            "meta": { "vct_values": ["urn:example:pid"] },
            "claim": [
                { "path": ["family_name"] },
                { "path": ["age"] }
            ]
        }));
        let partially_authorized = credential(json!({
            "format": "dc+sd-jwt",
            "meta": { "vct_values": ["urn:example:pid"] },
            "claim": [{ "path": ["family_name"] }]
        }));

        validate_query_authorization(&query, &[fully_authorized]).unwrap();
        assert_matches!(
            validate_query_authorization(&query, &[partially_authorized]),
            Err(RegistrationCertificateAuthorizationError::UnauthorizedCredential(_))
        );
    }

    #[rstest]
    #[case::unspecified(json!({
        "path": ["urn:example:pid", "family_name"]
    }))]
    #[case::not_retained(json!({
        "path": ["urn:example:pid", "family_name"],
        "intent_to_retain": false
    }))]
    #[case::retained(json!({
        "path": ["urn:example:pid", "family_name"],
        "intent_to_retain": true
    }))]
    fn authorize_mdoc_query_with_matching_doctype_and_claim(#[case] claim: Value) {
        let query = query(json!({
            "id": "pid",
            "format": "mso_mdoc",
            "meta": { "doctype_value": "urn:example:pid" },
            "claims": [claim]
        }));
        let authorized = credential(json!({
            "format": "mso_mdoc",
            "meta": { "doctype_value": "urn:example:pid" },
            "claim": [{ "path": ["urn:example:pid", "family_name"] }]
        }));

        validate_query_authorization(&query, &[authorized]).unwrap();
    }

    #[rstest]
    #[case::mismatching_mdoc_doctype(
        json!({
            "id": "pid",
            "format": "mso_mdoc",
            "meta": { "doctype_value": "urn:example:pid" },
            "claims": [{ "path": ["urn:example:pid", "family_name"] }]
        }),
        json!({
            "format": "mso_mdoc",
            "meta": { "doctype_value": "urn:example:address" },
            "claim": [{ "path": ["urn:example:pid", "family_name"] }]
        })
    )]
    #[case::different_credential_format(
        sd_jwt_query_value(
            "pid",
            &json!(["urn:example:pid"]),
            &json!([{ "path": ["family_name"] }])
        ),
        json!({
            "format": "mso_mdoc",
            "meta": { "doctype_value": "urn:example:pid" },
            "claim": [{ "path": ["family_name"] }]
        })
    )]
    fn reject_query_with_unauthorized_format(#[case] query_value: Value, #[case] authorized: Value) {
        let query = query(query_value);
        let authorized = credential(authorized);

        assert_matches!(
            validate_query_authorization(&query, &[authorized]),
            Err(RegistrationCertificateAuthorizationError::UnauthorizedCredential(_))
        );
    }
}
