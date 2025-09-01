use std::collections::HashMap;
use std::collections::HashSet;

use itertools::Itertools;

use dcql::CredentialFormat;
use dcql::normalized::AttributeRequest;
use dcql::normalized::NormalizedCredentialRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureType {
    Login,
    Regular,
}

impl DisclosureType {
    pub fn from_credential_requests<'a>(
        credential_requests: impl IntoIterator<Item = &'a NormalizedCredentialRequest>,
        login_attestation_types: &HashSet<&str>,
        login_claims: &HashMap<CredentialFormat, AttributeRequest>,
    ) -> Self {
        // Consider the disclosure type a login if there is only one credential request...
        credential_requests
            .into_iter()
            .exactly_one()
            .ok()
            .and_then(|request| {
                let login_claim = login_claims.get(&(&request.format).into());

                // ...and that request is for one of the formats for which we have login request, contains a subset of
                // the login attestation types and contains exactly the same requested claims as the login request.
                let is_login = login_claim.is_some()
                    && request
                        .format
                        .credential_types()
                        .collect::<HashSet<_>>()
                        .is_subset(login_attestation_types)
                    && request.claims.iter().exactly_one().ok() == login_claim;

                is_login.then_some(DisclosureType::Login)
            })
            .unwrap_or(DisclosureType::Regular)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::collections::HashSet;

    use rstest::rstest;

    use dcql::CredentialFormat;
    use dcql::normalized;
    use dcql::normalized::NormalizedCredentialRequests;

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

        let mdoc_login_claims =
            normalized::mock::mock_attribute_request_from_slice(&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]);
        let sd_jwt_login_claims = normalized::mock::mock_attribute_request_from_slice(&[LOGIN_ATTRIBUTE_ID]);
        let login_claims = HashMap::from([
            (CredentialFormat::MsoMdoc, mdoc_login_claims),
            (CredentialFormat::SdJwt, sd_jwt_login_claims),
        ]);

        assert_eq!(
            DisclosureType::from_credential_requests(attribute_paths.as_ref(), &login_attestation_types, &login_claims),
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
