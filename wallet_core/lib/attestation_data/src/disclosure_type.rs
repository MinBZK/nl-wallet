use std::collections::HashSet;

use itertools::Itertools;

use dcql::normalized::FormatCredentialRequest;
use dcql::normalized::MdocAttributeRequest;
use dcql::normalized::NormalizedCredentialRequest;
use dcql::normalized::SdJwtAttributeRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureType {
    Login,
    Regular,
}

impl DisclosureType {
    pub fn from_credential_requests<'a>(
        credential_requests: impl IntoIterator<Item = &'a NormalizedCredentialRequest>,
        login_attestation_types: &HashSet<&str>,
        mdoc_login_attribute: &MdocAttributeRequest,
        sd_jwt_login_attribute: &SdJwtAttributeRequest,
    ) -> Self {
        // Consider the disclosure type a login if there is only one credential request...
        credential_requests
            .into_iter()
            .exactly_one()
            .ok()
            .and_then(|request| {
                // ...and contains a subset of the login attestation types and contains exactly
                // the same requested attribute as the login request of the same format.
                match &request.format_request {
                    FormatCredentialRequest::MsoMdoc { doctype_value, claims } => {
                        login_attestation_types.contains(doctype_value.as_str())
                            && claims.iter().exactly_one().ok() == Some(mdoc_login_attribute)
                    }
                    FormatCredentialRequest::SdJwt { vct_values, claims } => {
                        vct_values
                            .iter()
                            .map(String::as_str)
                            .collect::<HashSet<_>>()
                            .is_subset(login_attestation_types)
                            && claims.iter().exactly_one().ok() == Some(sd_jwt_login_attribute)
                    }
                }
                .then_some(DisclosureType::Login)
            })
            .unwrap_or(DisclosureType::Regular)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use rstest::rstest;

    use attestation_types::claim_path::ClaimPath;
    use dcql::normalized::MdocAttributeRequest;
    use dcql::normalized::NormalizedCredentialRequests;
    use dcql::normalized::SdJwtAttributeRequest;
    use utils::vec_nonempty;

    use super::DisclosureType;

    const LOGIN_ATTESTATION_TYPE: &str = "pid";
    const ALSO_LOGIN_ATTESTATION_TYPE: &str = "also_pid";
    const LOGIN_NAMESPACE: &str = "pid";
    const LOGIN_ATTRIBUTE_ID: &str = "bsn";

    #[rstest]
    #[case(mdoc_pid_bsn_attribute_paths(), DisclosureType::Login)]
    #[case(mdoc_pid_bsn_and_other_attribute_paths(), DisclosureType::Regular)]
    #[case(mdoc_pid_and_other_bsn_attribute_paths(), DisclosureType::Regular)]
    #[case(mdoc_pid_too_short_attribute_paths(), DisclosureType::Regular)]
    #[case(mdoc_pid_too_long_attribute_paths(), DisclosureType::Regular)]
    #[case(sd_jwt_pid_bsn_attribute_paths(), DisclosureType::Login)]
    #[case(sd_jwt_double_pid_bsn_attribute_paths(), DisclosureType::Login)]
    #[case(sd_jwt_pid_bsn_and_other_attribute_paths(), DisclosureType::Regular)]
    #[case(sd_jwt_pid_and_other_bsn_attribute_paths(), DisclosureType::Regular)]
    #[case(sd_jwt_pid_too_long_attribute_paths(), DisclosureType::Regular)]
    fn test_disclosure_type_from_request_attribute_paths(
        #[case] attribute_paths: NormalizedCredentialRequests,
        #[case] expected: DisclosureType,
    ) {
        let login_attestation_types = HashSet::from([LOGIN_ATTESTATION_TYPE, ALSO_LOGIN_ATTESTATION_TYPE]);

        let mdoc_login_attribute = MdocAttributeRequest {
            path: vec_nonempty![
                ClaimPath::SelectByKey(LOGIN_NAMESPACE.to_string()),
                ClaimPath::SelectByKey(LOGIN_ATTRIBUTE_ID.to_string())
            ],
            intent_to_retain: None,
        };
        let sd_jwt_login_attribute = SdJwtAttributeRequest {
            path: vec_nonempty![ClaimPath::SelectByKey(LOGIN_ATTRIBUTE_ID.to_string())],
        };

        assert_eq!(
            DisclosureType::from_credential_requests(
                attribute_paths.as_ref(),
                &login_attestation_types,
                &mdoc_login_attribute,
                &sd_jwt_login_attribute
            ),
            expected
        );
    }

    fn mdoc_pid_bsn_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[(
            LOGIN_ATTESTATION_TYPE,
            &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]],
        )])
    }

    fn mdoc_pid_bsn_and_other_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[(
            LOGIN_ATTESTATION_TYPE,
            &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID], &[LOGIN_NAMESPACE, "other"]],
        )])
    }

    fn mdoc_pid_and_other_bsn_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[
            (LOGIN_ATTESTATION_TYPE, &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]]),
            ("other", &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]]),
        ])
    }

    fn mdoc_pid_too_short_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[(LOGIN_ATTESTATION_TYPE, &[&[LOGIN_ATTRIBUTE_ID]])])
    }

    fn mdoc_pid_too_long_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[(
            LOGIN_ATTESTATION_TYPE,
            &[&[LOGIN_NAMESPACE, LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]],
        )])
    }

    fn sd_jwt_pid_bsn_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[LOGIN_ATTESTATION_TYPE],
            &[&[LOGIN_ATTRIBUTE_ID]],
        )])
    }

    fn sd_jwt_double_pid_bsn_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[ALSO_LOGIN_ATTESTATION_TYPE, LOGIN_ATTESTATION_TYPE],
            &[&[LOGIN_ATTRIBUTE_ID]],
        )])
    }

    fn sd_jwt_pid_bsn_and_other_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[LOGIN_ATTESTATION_TYPE],
            &[&[LOGIN_ATTRIBUTE_ID], &["other"]],
        )])
    }

    fn sd_jwt_pid_and_other_bsn_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[
            (&[LOGIN_ATTESTATION_TYPE], &[&[LOGIN_ATTRIBUTE_ID]]),
            (&["other"], &[&[LOGIN_ATTRIBUTE_ID]]),
        ])
    }

    fn sd_jwt_pid_too_long_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[LOGIN_ATTESTATION_TYPE],
            &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]],
        )])
    }
}
