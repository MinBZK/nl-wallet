use std::hash::Hash;
use std::time::Duration;

use indexmap::IndexMap;
use semver::Op;
use semver::Version;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::DurationSeconds;

#[derive(Debug, thiserror::Error)]
pub enum VersionReqError {
    #[error("semver error: {0}")]
    Semver(#[from] semver::Error),
    #[error("operations Tilde and Caret are not supported")]
    TildeOrCaret,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionReq(semver::VersionReq);

impl VersionReq {
    // this version takes a reference, because the semver::VersionReq::matches function takes a reference
    pub fn matches(&self, version: &Version) -> bool {
        let mut version = version.clone();
        version.pre = semver::Prerelease::EMPTY;
        self.0.matches(&version)
    }

    pub fn parse(text: &str) -> Result<Self, VersionReqError> {
        let semver = semver::VersionReq::parse(text)?;
        if semver.comparators.iter().any(|c| matches!(c.op, Op::Tilde | Op::Caret)) {
            return Err(VersionReqError::TildeOrCaret);
        }
        Ok(Self(semver))
    }
}

// Actual state of a version, in reversed order of severity (s.t. the smallest Duration is the most sever)
#[serde_as]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, strum::Display)]
pub enum VersionState {
    Block,
    Warn(#[serde_as(as = "DurationSeconds<u64>")] Duration),
    Recommend,
    Notify,
    #[default]
    Ok,
}

// Response type for the update policy endpoint, does not contain any time information
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatePolicyResponse(pub IndexMap<VersionReq, VersionState>);

impl UpdatePolicyResponse {
    pub fn into_version_state(&self, version: &Version) -> VersionState {
        // find all states for the version and return the most severe one
        self.0
            .iter()
            .filter_map(|(range, state)| range.matches(version).then_some(*state))
            .min()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    #[rstest]
    #[case("=1.0.0", Ok(VersionReq(semver::VersionReq::parse("=1.0.0").unwrap())))]
    #[case(">1.0.0", Ok(VersionReq(semver::VersionReq::parse(">1.0.0").unwrap())))]
    #[case("<=1.0.0", Ok(VersionReq(semver::VersionReq::parse("<=1.0.0").unwrap())))]
    #[case("*", Ok(VersionReq(semver::VersionReq::STAR)))]
    #[case("1.0.0", Err(VersionReqError::TildeOrCaret))]
    #[case("~1.0.0", Err(VersionReqError::TildeOrCaret))]
    #[case("^1.0.0", Err(VersionReqError::TildeOrCaret))]
    #[case("1.0.0, ^2.0.0", Err(VersionReqError::TildeOrCaret))]
    #[case(">1.0.0, ~1.0.0", Err(VersionReqError::TildeOrCaret))]
    fn test_version_parse(#[case] s: &str, #[case] expected: Result<VersionReq, VersionReqError>) {
        let req = VersionReq::parse(s);
        match (req, expected) {
            (Ok(o), Ok(k)) => assert_eq!(o, k),
            // unfortunately some of the errors don't implement PartialEq
            (Err(e), Err(r)) => assert_eq!(e.to_string(), r.to_string()),
            (Err(e), Ok(o)) => {
                panic!("assertion `left == right` failed\n left: {e:?}\nright: {o:?}")
            }
            (Ok(o), Err(e)) => {
                panic!("assertion `left == right` failed\n left: {o:?}\nright: {e:?}")
            }
        };
    }

    #[rstest]
    #[case("<1.0.0", Version::new(0, 0, 0), Version::new(1, 0, 0))]
    #[case(">=0.0.0, <1.0.0", Version::new(0, 0, 0), Version::new(1, 0, 0))]
    #[case(">=0.0.0, <1.0.0", Version::parse("0.0.0-dev").unwrap(), Version::parse("1.0.0-dev").unwrap())]
    #[case(">=1.2.3, <=4.5.6", Version::new(2, 3, 4), Version::new(4, 5, 7))]
    #[case(">1.0.0, <2.0.0", Version::new(1, 0, 1), Version::new(2, 0, 1))]
    #[case("<=1.0.0", Version::new(1, 0, 0), Version::new(1, 0, 1))]
    #[case(">=2.0.0", Version::new(2, 0, 1), Version::new(1, 0, 0))]
    #[case("=2.0.0", Version::new(2, 0, 0), Version::new(2, 0, 1))]
    fn test_version_req_matches(#[case] s: &str, #[case] inn: Version, #[case] out: Version) {
        let req = VersionReq::parse(s).unwrap();
        assert!(req.matches(&inn));
        assert!(!req.matches(&out));
    }

    #[rstest]
    #[case(VersionState::Ok, VersionState::Notify)]
    #[case(VersionState::Ok, VersionState::Recommend)]
    #[case(VersionState::Ok, VersionState::Warn(Duration::default()))]
    #[case(VersionState::Ok, VersionState::Block)]
    #[case(VersionState::Notify, VersionState::Recommend)]
    #[case(VersionState::Notify, VersionState::Warn(Duration::default()))]
    #[case(VersionState::Notify, VersionState::Block)]
    #[case(VersionState::Recommend, VersionState::Warn(Duration::default()))]
    #[case(VersionState::Recommend, VersionState::Block)]
    #[case(VersionState::Warn(Duration::from_secs(1)), VersionState::Warn(Duration::default()))]
    #[case(VersionState::Warn(Duration::default()), VersionState::Block)]
    fn test_ord_versionstate(#[case] lhs: VersionState, #[case] rhs: VersionState) {
        assert!(lhs > rhs);
        assert!(rhs < lhs);
    }

    #[rstest]
    #[case(UpdatePolicyResponse(
        IndexMap::from([
            (VersionReq::parse(">=0.1.0, <1.0.0").unwrap(), VersionState::Warn(Duration::from_secs(4 * 24 * 60 * 60))),
            (VersionReq::parse("=0.5.0").unwrap(), VersionState::Recommend),
            (VersionReq::parse("=1.1.0").unwrap(), VersionState::Notify),
            (VersionReq::parse("=1.0.0").unwrap(), VersionState::Ok),
            (VersionReq::parse("=2.0.0").unwrap(), VersionState::Ok),
            (VersionReq::parse("=1.1.0").unwrap(), VersionState::Notify),
            (VersionReq::parse("=1.2.0").unwrap(), VersionState::Block),
        ])
    ), json!({
        "=0.5.0": "Recommend",
        "=1.0.0": "Ok",
        "=1.1.0": "Notify",
        "=1.2.0": "Block",
        "=2.0.0": "Ok",
        ">=0.1.0, <1.0.0": {
            "Warn": 345600,
        }
    }))]
    fn test_serialize_policy_response(#[case] policy: UpdatePolicyResponse, #[case] expected: serde_json::Value) {
        assert_eq!(serde_json::to_value(&policy).unwrap(), expected);
    }
}
