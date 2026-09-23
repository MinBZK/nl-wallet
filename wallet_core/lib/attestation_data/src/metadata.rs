use attestation_types::claim_path::ClaimPath;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

/// Describes the constraints of a single attestation claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimConstraint<'a> {
    /// The path to the claim within the attestation.
    pub path: &'a VecNonEmpty<ClaimPath>,

    /// Whether the issuer is required to include this claim in the attestation.
    pub mandatory: bool,
}

impl<'a> ClaimConstraint<'a> {
    /// The `SelectByKey` path components of this claim as keys.
    pub fn key_path(&self) -> Option<VecNonEmpty<&'a str>> {
        self.path
            .nonempty_iter()
            .map(|path| path.try_key_path().ok_or(()))
            .collect::<Result<VecNonEmpty<_>, _>>()
            .ok()
    }
}

/// Implemented by every kind of metadata that describes the claims an attestation may contain.
pub trait AttestationClaims {
    /// The claims this metadata describes, which together determine the attributes that an attestation is permitted
    /// and required to contain.
    fn claim_constraints(&self) -> impl Iterator<Item = ClaimConstraint<'_>>;
}

impl AttestationClaims for NormalizedTypeMetadata {
    fn claim_constraints(&self) -> impl Iterator<Item = ClaimConstraint<'_>> {
        self.claims().iter().map(|claim| ClaimConstraint {
            path: &claim.path,
            mandatory: claim.mandatory,
        })
    }
}

#[cfg(test)]
mod test {
    use attestation_types::claim_path::ClaimPath;
    use utils::vec_nonempty;

    use crate::metadata::ClaimConstraint;

    #[test]
    fn test_claim_constraints_key_path() {
        let birth_date_paths = ClaimPath::select_by_keys(&["birth_date"]);
        let locality_paths = ClaimPath::select_by_keys(&["place_of_birth", "locality"]);

        let claims = vec_nonempty![
            ClaimConstraint {
                path: &birth_date_paths,
                mandatory: true,
            },
            ClaimConstraint {
                path: &locality_paths,
                mandatory: false,
            }
        ];

        assert_eq!(
            claims
                .iter()
                .map(|claim| (claim.path.clone(), claim.mandatory))
                .collect::<Vec<_>>(),
            vec![
                (ClaimPath::select_by_keys(&["birth_date"]), true),
                (ClaimPath::select_by_keys(&["place_of_birth", "locality"]), false),
            ]
        );
        assert_eq!(
            claims.iter().filter_map(ClaimConstraint::key_path).collect::<Vec<_>>(),
            vec![vec_nonempty!["birth_date"], vec_nonempty!["place_of_birth", "locality"],]
        );
    }
}
