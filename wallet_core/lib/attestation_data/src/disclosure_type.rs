use std::collections::HashSet;

use itertools::Itertools;

use attestation_types::claim_path::ClaimPath;
use dcql::normalized::FormatCredentialRequest;
use dcql::normalized::NormalizedCredentialRequest;

use crate::constants::PID_BSN;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureType {
    Login,
    Regular,
}

impl DisclosureType {
    pub fn from_credential_requests<'a>(
        credential_requests: impl IntoIterator<Item = &'a NormalizedCredentialRequest>,
        login_attestation_types: &HashSet<&str>,
    ) -> Self {
        // Consider the disclosure type a login if there is only one credential request...
        credential_requests
            .into_iter()
            .exactly_one()
            .ok()
            .and_then(|request| {
                match &request.format_request {
                    // ...and for mdoc the doc type is one of the known login types, there is only one claim
                    // requested and that claim is the BSN and will not be retained by the verifier...
                    FormatCredentialRequest::MsoMdoc { doctype_value, claims } => {
                        login_attestation_types.contains(doctype_value.as_str())
                            && claims.iter().exactly_one().ok().is_some_and(|attribute_request| {
                                ClaimPath::matches_key_path(&attribute_request.path, [doctype_value.as_str(), PID_BSN])
                                    && attribute_request.intent_to_retain == Some(false)
                            })
                    }
                    // ...or for SD-JWT the VCTs are a subset of the known login types,
                    // there is only one claim requested and that claim is the BSN.
                    FormatCredentialRequest::SdJwt { vct_values, claims } => {
                        vct_values
                            .iter()
                            .map(String::as_str)
                            .collect::<HashSet<_>>()
                            .is_subset(login_attestation_types)
                            && claims.iter().exactly_one().ok().is_some_and(|attribute_request| {
                                ClaimPath::matches_key_path(&attribute_request.path, [PID_BSN])
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
    use std::collections::HashSet;

    use rstest::rstest;

    use dcql::normalized::NormalizedCredentialRequests;

    use crate::constants::PID_BSN;

    use super::DisclosureType;

    const LOGIN_ATTESTATION_TYPE: &str = "pid";
    const ALSO_LOGIN_ATTESTATION_TYPE: &str = "also_pid";

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

        assert_eq!(
            DisclosureType::from_credential_requests(attribute_paths.as_ref(), &login_attestation_types,),
            expected
        );
    }

    fn mdoc_pid_bsn_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(
            &[(LOGIN_ATTESTATION_TYPE, &[&[LOGIN_ATTESTATION_TYPE, PID_BSN]])],
            Some(false),
        )
    }

    fn mdoc_pid_bsn_and_other_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(
            &[(
                LOGIN_ATTESTATION_TYPE,
                &[&[LOGIN_ATTESTATION_TYPE, PID_BSN], &[LOGIN_ATTESTATION_TYPE, "other"]],
            )],
            Some(false),
        )
    }

    fn mdoc_pid_and_other_bsn_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(
            &[
                (LOGIN_ATTESTATION_TYPE, &[&[LOGIN_ATTESTATION_TYPE, PID_BSN]]),
                ("other", &[&[LOGIN_ATTESTATION_TYPE, PID_BSN]]),
            ],
            Some(false),
        )
    }

    fn mdoc_pid_too_short_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[(LOGIN_ATTESTATION_TYPE, &[&[PID_BSN]])], Some(false))
    }

    fn mdoc_pid_too_long_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(
            &[(
                LOGIN_ATTESTATION_TYPE,
                &[&[LOGIN_ATTESTATION_TYPE, LOGIN_ATTESTATION_TYPE, PID_BSN]],
            )],
            Some(false),
        )
    }

    fn sd_jwt_pid_bsn_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(&[LOGIN_ATTESTATION_TYPE], &[&[PID_BSN]])])
    }

    fn sd_jwt_double_pid_bsn_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[ALSO_LOGIN_ATTESTATION_TYPE, LOGIN_ATTESTATION_TYPE],
            &[&[PID_BSN]],
        )])
    }

    fn sd_jwt_pid_bsn_and_other_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[LOGIN_ATTESTATION_TYPE],
            &[&[PID_BSN], &["other"]],
        )])
    }

    fn sd_jwt_pid_and_other_bsn_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[
            (&[LOGIN_ATTESTATION_TYPE], &[&[PID_BSN]]),
            (&["other"], &[&[PID_BSN]]),
        ])
    }

    fn sd_jwt_pid_too_long_attribute_paths() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[LOGIN_ATTESTATION_TYPE],
            &[&[LOGIN_ATTESTATION_TYPE, PID_BSN]],
        )])
    }
}
