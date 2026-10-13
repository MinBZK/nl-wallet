use crate::models::attestation::AttestationPresentation;
use crate::models::organization::Organization;

pub enum IssuanceStartResult {
    AuthorizationUrl(String),
    Previews(Vec<AttestationPresentation>),
    NoPreviews { organization: Organization },
}

impl From<wallet::IssuanceStartResult> for IssuanceStartResult {
    fn from(source: wallet::IssuanceStartResult) -> IssuanceStartResult {
        use wallet::IssuanceStartResult::*;
        match source {
            AuthorizationUrl(url) => IssuanceStartResult::AuthorizationUrl(url.into()),
            Previews(previews) => {
                IssuanceStartResult::Previews(previews.into_iter().map(AttestationPresentation::from).collect())
            }
            NoPreviews { organization } => IssuanceStartResult::NoPreviews {
                organization: (*organization).into(),
            },
        }
    }
}

pub enum IssuanceContinueResult {
    Previews(Vec<AttestationPresentation>),
    NoPreviews { organization: Organization },
}

impl From<wallet::IssuanceContinueResult> for IssuanceContinueResult {
    fn from(source: wallet::IssuanceContinueResult) -> IssuanceContinueResult {
        use wallet::IssuanceContinueResult::*;
        match source {
            Previews(previews) => {
                IssuanceContinueResult::Previews(previews.into_iter().map(AttestationPresentation::from).collect())
            }
            NoPreviews { organization } => IssuanceContinueResult::NoPreviews {
                organization: (*organization).into(),
            },
        }
    }
}
