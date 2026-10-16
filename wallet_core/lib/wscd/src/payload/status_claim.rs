use attestation_types::status_claim::StatusClaim;
use derive_more::Constructor;
use serde::Deserialize;
use serde::Serialize;
use utils::date_time_seconds::DateTimeSeconds;

/// A status list reference and maintenance period for either a Wallet Instance or the WSCD key storage.
///
/// Source: <https://github.com/eu-digital-identity-wallet/eudi-doc-standards-and-technical-specifications/blob/5924eb77ab4495d4dc0a874e54ac3e5de1fbd5b1/docs/technical-specifications/ts3-wallet-unit-attestation.md#231-content-of-wia>
/// and <https://github.com/eu-digital-identity-wallet/eudi-doc-standards-and-technical-specifications/blob/5924eb77ab4495d4dc0a874e54ac3e5de1fbd5b1/docs/technical-specifications/ts3-wallet-unit-attestation.md#232-content-of-key-attestation-ka>
#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct TimeBoundedStatusClaim {
    // A status list reference as specified in Appendix D.1 of OID4VCI. The value represents either the revocation
    // state of the Wallet Instance, WSCD key storage or WSCD key storage type.
    pub status: StatusClaim,

    /// A NumericDate (as defined in RFC 7519) specifying the time until which the Wallet Provider commits to
    /// maintaining the revocation status at the status list index referenced in `status`.
    pub exp: DateTimeSeconds,
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use attestation_types::status_claim::StatusClaim;
    use chrono::Duration;
    use chrono::Utc;

    use super::TimeBoundedStatusClaim;

    impl TimeBoundedStatusClaim {
        pub fn new_mock() -> Self {
            Self::new(StatusClaim::new_mock(), (Utc::now() + Duration::days(365)).into())
        }
    }
}
