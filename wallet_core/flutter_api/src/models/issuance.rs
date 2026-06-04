use url::Url;

use crate::models::attestation::AttestationPresentation;

pub enum IssuanceStartResult {
    AuthorizationUrl(Url),
    Previews(Vec<AttestationPresentation>),
}

impl From<wallet::IssuanceStartResult> for IssuanceStartResult {
    fn from(source: wallet::IssuanceStartResult) -> IssuanceStartResult {
        use wallet::IssuanceStartResult::*;
        match source {
            AuthorizationUrl(url) => IssuanceStartResult::AuthorizationUrl(url),
            Previews(previews) => {
                IssuanceStartResult::Previews(previews.into_iter().map(AttestationPresentation::from).collect())
            }
        }
    }
}
