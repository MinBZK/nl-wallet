use itertools::Itertools;

use attestation_types::claim_path::ClaimPath;
use dcql::normalized::NormalizedCredentialRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureType {
    Login,
    Regular,
}

pub trait DisclosureTypeConfig {
    fn mdoc_login_path(&self, doctype: &str) -> Option<impl Iterator<Item = &str>>;
    fn sd_jwt_login_path(&self, vct: &str) -> Option<impl Iterator<Item = &str>>;
}

impl DisclosureType {
    pub fn from_credential_requests<'a>(
        credential_requests: impl IntoIterator<Item = &'a NormalizedCredentialRequest>,
        config: &impl DisclosureTypeConfig,
    ) -> Self {
        // Consider the disclosure type a login if there is only one credential request...
        credential_requests
            .into_iter()
            .exactly_one()
            .ok()
            .and_then(|request| {
                match request {
                    // ...and for mdoc the doc type is one of the known login types, there is only one claim
                    // requested and that claim is the login path and will not be retained by the verifier...
                    NormalizedCredentialRequest::MsoMdoc {
                        doctype_value, claims, ..
                    } => claims.iter().exactly_one().ok().is_some_and(|attribute_request| {
                        attribute_request.intent_to_retain == Some(false)
                            && config
                                .mdoc_login_path(doctype_value)
                                .is_some_and(|path| ClaimPath::matches_key_path(&attribute_request.path, path))
                    }),
                    // ...or for SD-JWT the VCTs are a subset of the known login types,
                    // there is only one claim requested and that claim is the login path.
                    NormalizedCredentialRequest::SdJwt { vct_values, claims, .. } => {
                        claims.iter().exactly_one().ok().is_some_and(|attribute_request| {
                            vct_values
                                .iter()
                                .map(|vct| config.sd_jwt_login_path(vct))
                                .collect::<Option<Vec<_>>>()
                                // Using `any` which means that we could generate a false positive if the configured
                                // paths are different between the VCTs. Note that whether this is a login flow is in
                                // this case dependent on the VCT of the response, which we do not know here yet.
                                .is_some_and(|paths| {
                                    paths
                                        .into_iter()
                                        .any(|path| ClaimPath::matches_key_path(&attribute_request.path, path))
                                })
                        })
                    }
                }
                .then_some(DisclosureType::Login)
            })
            .unwrap_or(DisclosureType::Regular)
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use dcql::normalized::NormalizedCredentialRequests;

    use super::DisclosureType;
    use super::DisclosureTypeConfig;

    const LOGIN_ATTESTATION_TYPE: &str = "pid";
    const ALSO_LOGIN_ATTESTATION_TYPE: &str = "also_pid";

    const LOGIN_CLAIM_PATH: &str = "login";

    const DIFFERENT_LOGIN_ATTESTATION_TYPE: &str = "different_pid";

    const DIFFERENT_LOGIN_PATH: &str = "different_login";

    struct LoginAttestationConfig;

    impl DisclosureTypeConfig for LoginAttestationConfig {
        fn mdoc_login_path(&self, doctype: &str) -> Option<impl Iterator<Item = &str>> {
            match doctype {
                LOGIN_ATTESTATION_TYPE => Some([LOGIN_ATTESTATION_TYPE, LOGIN_CLAIM_PATH].iter().copied()),
                ALSO_LOGIN_ATTESTATION_TYPE => Some([ALSO_LOGIN_ATTESTATION_TYPE, LOGIN_CLAIM_PATH].iter().copied()),
                _ => None,
            }
        }

        fn sd_jwt_login_path(&self, vct: &str) -> Option<impl Iterator<Item = &str>> {
            match vct {
                LOGIN_ATTESTATION_TYPE | ALSO_LOGIN_ATTESTATION_TYPE => Some([LOGIN_CLAIM_PATH].iter().copied()),
                DIFFERENT_LOGIN_ATTESTATION_TYPE => Some([DIFFERENT_LOGIN_PATH].iter().copied()),
                _ => None,
            }
        }
    }

    #[rstest]
    #[case(mdoc_pid_login_attribute_paths(), DisclosureType::Login)]
    #[case(mdoc_pid_login_and_other_attribute_paths(), DisclosureType::Regular)]
    #[case(mdoc_pid_and_other_login_attribute_paths(), DisclosureType::Regular)]
    #[case(mdoc_pid_too_short_attribute_paths(), DisclosureType::Regular)]
    #[case(mdoc_pid_too_long_attribute_paths(), DisclosureType::Regular)]
    #[case(sd_jwt_pid_login_attribute_paths(), DisclosureType::Login)]
    #[case(sd_jwt_double_pid_login_attribute_paths(), DisclosureType::Login)]
    #[case(sd_jwt_pid_login_other_attribute_type(), DisclosureType::Regular)]
    #[case(sd_jwt_pid_login_with_different_login_type(), DisclosureType::Login)]
    #[case(sd_jwt_pid_login_and_other_attribute_paths(), DisclosureType::Regular)]
    #[case(sd_jwt_pid_and_other_login_attribute_paths(), DisclosureType::Regular)]
    #[case(sd_jwt_pid_too_long_attribute_paths(), DisclosureType::Regular)]
    fn test_disclosure_type_from_request_attribute_paths(
        #[case] attribute_paths: NormalizedCredentialRequests,
        #[case] expected: DisclosureType,
    ) {
        let config = LoginAttestationConfig {};

        assert_eq!(
            DisclosureType::from_credential_requests(attribute_paths.as_ref(), &config),
            expected
        );
    }

    fn mdoc_pid_login_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(
            &[(LOGIN_ATTESTATION_TYPE, &[&[LOGIN_ATTESTATION_TYPE, LOGIN_CLAIM_PATH]])],
            Some(false),
        )
    }

    fn mdoc_pid_login_and_other_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(
            &[(
                LOGIN_ATTESTATION_TYPE,
                &[
                    &[LOGIN_ATTESTATION_TYPE, LOGIN_CLAIM_PATH],
                    &[LOGIN_ATTESTATION_TYPE, "other"],
                ],
            )],
            Some(false),
        )
    }

    fn mdoc_pid_and_other_login_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(
            &[
                (LOGIN_ATTESTATION_TYPE, &[&[LOGIN_ATTESTATION_TYPE, LOGIN_CLAIM_PATH]]),
                ("other", &[&[LOGIN_ATTESTATION_TYPE, LOGIN_CLAIM_PATH]]),
            ],
            Some(false),
        )
    }

    fn mdoc_pid_too_short_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(
            &[(LOGIN_ATTESTATION_TYPE, &[&[LOGIN_CLAIM_PATH]])],
            Some(false),
        )
    }

    fn mdoc_pid_too_long_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(
            &[(
                LOGIN_ATTESTATION_TYPE,
                &[&[LOGIN_ATTESTATION_TYPE, LOGIN_ATTESTATION_TYPE, LOGIN_CLAIM_PATH]],
            )],
            Some(false),
        )
    }

    fn sd_jwt_pid_login_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[LOGIN_ATTESTATION_TYPE],
            &[&[LOGIN_CLAIM_PATH]],
        )])
    }

    fn sd_jwt_double_pid_login_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[ALSO_LOGIN_ATTESTATION_TYPE, LOGIN_ATTESTATION_TYPE],
            &[&[LOGIN_CLAIM_PATH]],
        )])
    }

    fn sd_jwt_pid_login_other_attribute_type() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[LOGIN_ATTESTATION_TYPE, "other"],
            &[&[LOGIN_CLAIM_PATH]],
        )])
    }

    fn sd_jwt_pid_login_with_different_login_type() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[LOGIN_ATTESTATION_TYPE, DIFFERENT_LOGIN_ATTESTATION_TYPE],
            &[&[LOGIN_CLAIM_PATH]],
        )])
    }

    fn sd_jwt_pid_login_and_other_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[LOGIN_ATTESTATION_TYPE],
            &[&[LOGIN_CLAIM_PATH], &["other"]],
        )])
    }

    fn sd_jwt_pid_and_other_login_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[
            (&[LOGIN_ATTESTATION_TYPE], &[&[LOGIN_CLAIM_PATH]]),
            (&["other"], &[&[LOGIN_CLAIM_PATH]]),
        ])
    }

    fn sd_jwt_pid_too_long_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[LOGIN_ATTESTATION_TYPE],
            &[&[LOGIN_ATTESTATION_TYPE, LOGIN_CLAIM_PATH]],
        )])
    }
}
