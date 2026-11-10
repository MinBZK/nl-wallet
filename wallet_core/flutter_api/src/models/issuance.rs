use crate::models::attestation::AttestationPresentation;
use crate::models::organization::Organization;

pub enum IssuanceStartResult {
    AuthorizationUrl(String),
    Previews(Vec<AttestationPresentation>),
    NoPreviews { issuer: Organization },
}

impl From<wallet::IssuanceStartResult> for IssuanceStartResult {
    fn from(source: wallet::IssuanceStartResult) -> IssuanceStartResult {
        use wallet::IssuanceStartResult::*;
        match source {
            AuthorizationUrl(url) => IssuanceStartResult::AuthorizationUrl(url.into()),
            Previews(previews) => {
                IssuanceStartResult::Previews(previews.into_iter().map(AttestationPresentation::from).collect())
            }
            NoPreviews { issuer } => IssuanceStartResult::NoPreviews {
                issuer: (*issuer).into(),
            },
        }
    }
}

pub enum IssuanceContinueResult {
    Previews(Vec<AttestationPresentation>),
    NoPreviews { issuer: Organization },
}

impl From<wallet::IssuanceContinueResult> for IssuanceContinueResult {
    fn from(source: wallet::IssuanceContinueResult) -> IssuanceContinueResult {
        use wallet::IssuanceContinueResult::*;
        match source {
            Previews(previews) => {
                IssuanceContinueResult::Previews(previews.into_iter().map(AttestationPresentation::from).collect())
            }
            NoPreviews { issuer } => IssuanceContinueResult::NoPreviews {
                issuer: (*issuer).into(),
            },
        }
    }
}

pub enum DisclosureBasedIssuanceResult {
    NoIssuableAttestations,
    Previews(Vec<AttestationPresentation>),
    NoPreviews { issuer: Organization },
}

impl From<wallet::DisclosureBasedIssuanceResult> for DisclosureBasedIssuanceResult {
    fn from(value: wallet::DisclosureBasedIssuanceResult) -> Self {
        match value {
            wallet::DisclosureBasedIssuanceResult::NoIssuableAttestations => {
                DisclosureBasedIssuanceResult::NoIssuableAttestations
            }
            wallet::DisclosureBasedIssuanceResult::Previews(attestations) => DisclosureBasedIssuanceResult::Previews(
                attestations.into_iter().map(AttestationPresentation::from).collect(),
            ),
            wallet::DisclosureBasedIssuanceResult::NoPreviews { issuer } => DisclosureBasedIssuanceResult::NoPreviews {
                issuer: (*issuer).into(),
            },
        }
    }
}
