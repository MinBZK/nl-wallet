use itertools::Itertools;

use dcql::CredentialQueryFormat;
use dcql::normalized::NormalizedCredentialRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureType {
    Login,
    Regular,
}

impl DisclosureType {
    pub fn from_credential_requests<'a>(
        credential_requests: impl IntoIterator<Item = &'a NormalizedCredentialRequest>,
        mdoc_login_request: &NormalizedCredentialRequest,
    ) -> Self {
        credential_requests
            .into_iter()
            .exactly_one()
            .ok()
            .and_then(|request| {
                match request.format {
                    CredentialQueryFormat::MsoMdoc { .. } => request == mdoc_login_request,
                    // TODO (PVW-4621): Add support for matching SDW-JWT login request.
                    CredentialQueryFormat::SdJwt { .. } => false,
                }
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
    const LOGIN_NAMESPACE: &str = "pid";
    const LOGIN_ATTRIBUTE_ID: &str = "bsn";

    #[rstest]
    #[case(pid_bsn_attribute_paths(), DisclosureType::Login)]
    #[case(pid_bsn_and_other_attribute_paths(), DisclosureType::Regular)]
    #[case(pid_and_other_bsn_attribute_paths(), DisclosureType::Regular)]
    #[case(pid_too_long_attribute_paths(), DisclosureType::Regular)]
    fn test_disclosure_type_from_request_attribute_paths(
        #[case] attribute_paths: VecNonEmpty<NormalizedCredentialRequest>,
        #[case] expected: DisclosureType,
    ) {
        let mdoc_login_request = normalized::mock::mock_mdoc_from_slices(&[(
            LOGIN_ATTESTATION_TYPE,
            &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]],
        )])
        .into_first();

        assert_eq!(
            DisclosureType::from_credential_requests(attribute_paths.as_ref(), &mdoc_login_request),
            expected
        );
    }

    fn pid_bsn_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_mdoc_from_slices(&[(LOGIN_ATTESTATION_TYPE, &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]])])
    }

    fn pid_bsn_and_other_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_mdoc_from_slices(&[(
            LOGIN_ATTESTATION_TYPE,
            &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID], &[LOGIN_NAMESPACE, "other"]],
        )])
    }

    fn pid_and_other_bsn_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_mdoc_from_slices(&[
            (LOGIN_ATTESTATION_TYPE, &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]]),
            ("other", &[&[LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]]),
        ])
    }

    fn pid_too_long_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_mdoc_from_slices(&[(
            LOGIN_ATTESTATION_TYPE,
            &[&[LOGIN_NAMESPACE, LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID]],
        )])
    }
}
