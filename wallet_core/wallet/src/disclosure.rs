use itertools::Itertools;

use attestation_types::disclosure::RequestedAttributePaths;

use crate::issuance::BSN_ATTR_NAME;
use crate::issuance::PID_DOCTYPE;
use crate::storage::DisclosureType;

/// Something is a login flow if the requested attributes has exactly one element,
/// which is of attestation type `PID_DOCTYPE` and path `[PID_DOCTYPE, BSN_ATTR_NAME]`.
pub fn disclosure_type_for_requested_attribute_paths(attribute_paths: &RequestedAttributePaths) -> DisclosureType {
    attribute_paths
        .as_ref()
        .keys()
        .exactly_one()
        .ok()
        .and_then(|attestation_type| {
            (attestation_type == PID_DOCTYPE).then(|| attribute_paths.as_mdoc_paths(PID_DOCTYPE))
        })
        .and_then(|paths| paths.into_iter().exactly_one().ok())
        .and_then(|path| (path == (PID_DOCTYPE, BSN_ATTR_NAME)).then_some(DisclosureType::Login))
        .unwrap_or(DisclosureType::Regular)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::collections::HashSet;

    use rstest::rstest;

    use utils::vec_at_least::VecNonEmpty;

    use super::*;

    #[rstest]
    #[case(pid_bsn_attribute_paths(), DisclosureType::Login)]
    #[case(pid_bsn_and_other_attribute_paths(), DisclosureType::Regular)]
    #[case(pid_and_other_bsn_attribute_paths(), DisclosureType::Regular)]
    #[case(pid_too_long_attribute_paths(), DisclosureType::Regular)]
    fn test_disclosure_type_from_proposed_attributes(
        #[case] attribute_paths: RequestedAttributePaths,
        #[case] expected: DisclosureType,
    ) {
        assert_eq!(
            disclosure_type_for_requested_attribute_paths(&attribute_paths),
            expected
        );
    }

    fn pid_bsn_attribute_paths() -> RequestedAttributePaths {
        RequestedAttributePaths::try_new(HashMap::from([(
            PID_DOCTYPE.to_string(),
            HashSet::from([VecNonEmpty::try_from(vec![PID_DOCTYPE.to_string(), BSN_ATTR_NAME.to_string()]).unwrap()]),
        )]))
        .unwrap()
    }

    fn pid_bsn_and_other_attribute_paths() -> RequestedAttributePaths {
        RequestedAttributePaths::try_new(HashMap::from([(
            PID_DOCTYPE.to_string(),
            HashSet::from([
                VecNonEmpty::try_from(vec![PID_DOCTYPE.to_string(), BSN_ATTR_NAME.to_string()]).unwrap(),
                VecNonEmpty::try_from(vec![PID_DOCTYPE.to_string(), "other".to_string()]).unwrap(),
            ]),
        )]))
        .unwrap()
    }

    fn pid_and_other_bsn_attribute_paths() -> RequestedAttributePaths {
        RequestedAttributePaths::try_new(HashMap::from([
            (
                PID_DOCTYPE.to_string(),
                HashSet::from([
                    VecNonEmpty::try_from(vec![PID_DOCTYPE.to_string(), BSN_ATTR_NAME.to_string()]).unwrap(),
                ]),
            ),
            (
                "other".to_string(),
                HashSet::from([
                    VecNonEmpty::try_from(vec![PID_DOCTYPE.to_string(), BSN_ATTR_NAME.to_string()]).unwrap(),
                ]),
            ),
        ]))
        .unwrap()
    }

    fn pid_too_long_attribute_paths() -> RequestedAttributePaths {
        RequestedAttributePaths::try_new(HashMap::from([(
            PID_DOCTYPE.to_string(),
            HashSet::from([VecNonEmpty::try_from(vec![
                PID_DOCTYPE.to_string(),
                PID_DOCTYPE.to_string(),
                BSN_ATTR_NAME.to_string(),
            ])
            .unwrap()]),
        )]))
        .unwrap()
    }
}
