use itertools::Itertools;

use dcql::CredentialQueryFormat;
use dcql::normalized::NormalizedCredentialRequest;
use mdoc::holder::disclosure::credential_requests_to_mdoc_paths;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureType {
    Login,
    Regular,
}

impl DisclosureType {
    pub fn from_request_attribute_paths(
        credential_requests: &VecNonEmpty<NormalizedCredentialRequest>,
        login_attestation_type: &str,
        login_attribute_path: (&str, &str),
    ) -> Self {
        credential_requests
            .as_ref()
            .iter()
            .exactly_one()
            .ok()
            .and_then(|credential_request| {
                (credential_request.format
                    == CredentialQueryFormat::MsoMdoc {
                        doctype_value: login_attestation_type.to_string(),
                    })
                .then(|| credential_requests_to_mdoc_paths(credential_requests, login_attestation_type))
            })
            .and_then(|paths| paths.into_iter().exactly_one().ok())
            .and_then(|path| (path == login_attribute_path).then_some(DisclosureType::Login))
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
        assert_eq!(
            DisclosureType::from_request_attribute_paths(
                &attribute_paths,
                LOGIN_ATTESTATION_TYPE,
                (LOGIN_NAMESPACE, LOGIN_ATTRIBUTE_ID)
            ),
            expected
        );
    }

    fn pid_bsn_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_from_vecs(vec![(
            LOGIN_ATTESTATION_TYPE.to_string(),
            vec![VecNonEmpty::try_from(vec![LOGIN_NAMESPACE.to_string(), LOGIN_ATTRIBUTE_ID.to_string()]).unwrap()],
        )])
    }

    fn pid_bsn_and_other_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_from_vecs(vec![(
            LOGIN_ATTESTATION_TYPE.to_string(),
            vec![
                VecNonEmpty::try_from(vec![LOGIN_NAMESPACE.to_string(), LOGIN_ATTRIBUTE_ID.to_string()]).unwrap(),
                VecNonEmpty::try_from(vec![LOGIN_NAMESPACE.to_string(), "other".to_string()]).unwrap(),
            ],
        )])
    }

    fn pid_and_other_bsn_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_from_vecs(vec![
            (
                LOGIN_ATTESTATION_TYPE.to_string(),
                vec![VecNonEmpty::try_from(vec![LOGIN_NAMESPACE.to_string(), LOGIN_ATTRIBUTE_ID.to_string()]).unwrap()],
            ),
            (
                "other".to_string(),
                vec![VecNonEmpty::try_from(vec![LOGIN_NAMESPACE.to_string(), LOGIN_ATTRIBUTE_ID.to_string()]).unwrap()],
            ),
        ])
    }

    fn pid_too_long_attribute_paths() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_from_vecs(vec![(
            LOGIN_ATTESTATION_TYPE.to_string(),
            vec![
                VecNonEmpty::try_from(vec![
                    LOGIN_NAMESPACE.to_string(),
                    LOGIN_NAMESPACE.to_string(),
                    LOGIN_ATTRIBUTE_ID.to_string(),
                ])
                .unwrap(),
            ],
        )])
    }
}
