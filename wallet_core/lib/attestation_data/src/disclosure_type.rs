use std::collections::HashSet;

use itertools::Itertools;

use dcql::CredentialQueryFormat;
use dcql::normalized::NormalizedCredentialRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureType {
    Login,
    Regular,
}

impl DisclosureType {
    /// Determine the discloure type based on received credential requests
    /// and a set of pre-determined login request templates.
    pub fn from_credential_requests<'a, 'b>(
        credential_requests: impl IntoIterator<Item = &'a NormalizedCredentialRequest>,
        login_requests: impl IntoIterator<Item = &'b NormalizedCredentialRequest>,
    ) -> Self {
        // Consider the disclosure type a login if there is only one credential request...
        credential_requests
            .into_iter()
            .exactly_one()
            .ok()
            .and_then(|request| {
                let request_attestation_types = request.format.attestation_types().collect::<HashSet<_>>();

                login_requests
                    .into_iter()
                    .filter(|login_request| {
                        // ...has the same format as at least one of the login templates...
                        matches!(
                            (&request.format, &login_request.format),
                            (
                                CredentialQueryFormat::MsoMdoc { .. },
                                CredentialQueryFormat::MsoMdoc { .. }
                            ) | (CredentialQueryFormat::SdJwt { .. }, CredentialQueryFormat::SdJwt { .. })
                        )
                    })
                    .any(|login_request| {
                        let login_attestation_types = login_request.format.attestation_types().collect::<HashSet<_>>();

                        // ...and the request contains a subset of that template's attestation types
                        // and contains exactly the same requested claims.
                        request_attestation_types.is_subset(&login_attestation_types)
                            && request.claims == login_request.claims
                    })
                    .then_some(DisclosureType::Login)
            })
            .unwrap_or(DisclosureType::Regular)
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use dcql::normalized;
    use utils::vec_at_least::VecNonEmpty;

    use super::*;

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
        #[case] attribute_paths: VecNonEmpty<NormalizedCredentialRequest>,
        #[case] expected: DisclosureType,
    ) {
        let mdoc_login_request = normalized::mock::mock_mdoc_from_slices(&[(
            LOGIN_ATTESTATION_TYPE,
            &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]],
        )])
        .into_first();

        let sd_jwt_login_request = normalized::mock::mock_sd_jwt_from_slices(&[(
            &[LOGIN_ATTESTATION_TYPE, ALSO_LOGIN_ATTESTATION_TYPE],
            &[&[LOGIN_ATTRIBUTE_ID]],
        )])
        .into_first();

        assert_eq!(
            DisclosureType::from_credential_requests(
                attribute_paths.as_ref(),
                &[mdoc_login_request, sd_jwt_login_request]
            ),
            expected
        );
    }

    fn mdoc_pid_bsn_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_mdoc_from_slices(&[(LOGIN_ATTESTATION_TYPE, &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]])])
    }

    fn mdoc_pid_bsn_and_other_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_mdoc_from_slices(&[(
            LOGIN_ATTESTATION_TYPE,
            &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID], &[LOGIN_NAMESPACE, "other"]],
        )])
    }

    fn mdoc_pid_and_other_bsn_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_mdoc_from_slices(&[
            (LOGIN_ATTESTATION_TYPE, &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]]),
            ("other", &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]]),
        ])
    }

    fn mdoc_pid_too_short_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_mdoc_from_slices(&[(LOGIN_ATTESTATION_TYPE, &[&[LOGIN_ATTRIBUTE_ID]])])
    }

    fn mdoc_pid_too_long_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_mdoc_from_slices(&[(
            LOGIN_ATTESTATION_TYPE,
            &[&[LOGIN_NAMESPACE, LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]],
        )])
    }

    fn sd_jwt_pid_bsn_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_sd_jwt_from_slices(&[(&[LOGIN_ATTESTATION_TYPE], &[&[LOGIN_ATTRIBUTE_ID]])])
    }

    fn sd_jwt_double_pid_bsn_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_sd_jwt_from_slices(&[(
            &[ALSO_LOGIN_ATTESTATION_TYPE, LOGIN_ATTESTATION_TYPE],
            &[&[LOGIN_ATTRIBUTE_ID]],
        )])
    }

    fn sd_jwt_pid_bsn_and_other_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_sd_jwt_from_slices(&[(&[LOGIN_ATTESTATION_TYPE], &[&[LOGIN_ATTRIBUTE_ID], &["other"]])])
    }

    fn sd_jwt_pid_and_other_bsn_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_sd_jwt_from_slices(&[
            (&[LOGIN_ATTESTATION_TYPE], &[&[LOGIN_ATTRIBUTE_ID]]),
            (&["other"], &[&[LOGIN_ATTRIBUTE_ID]]),
        ])
    }

    fn sd_jwt_pid_too_long_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_sd_jwt_from_slices(&[(
            &[LOGIN_ATTESTATION_TYPE],
            &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]],
        )])
    }
}
