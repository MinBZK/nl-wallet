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
