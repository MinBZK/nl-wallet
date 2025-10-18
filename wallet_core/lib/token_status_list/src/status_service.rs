use std::num::NonZeroUsize;

use uuid::Uuid;

use http_utils::urls::BaseUrl;
use utils::date_time_seconds::DateTimeSeconds;
use utils::vec_at_least::VecNonEmpty;

use crate::status_claim::StatusClaim;

#[trait_variant::make(Send)]
pub trait StatusClaimService {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn obtain_status_claims(
        &self,
        attestation_type: &str,
        batch_id: Uuid,
        base_url: BaseUrl,
        expires: Option<DateTimeSeconds>,
        copies: NonZeroUsize,
    ) -> Result<VecNonEmpty<StatusClaim>, Self::Error>;
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::convert::Infallible;

    use dashmap::DashMap;

    use http_utils::urls::BaseUrl;

    use crate::status_claim::StatusListClaim;

    use super::*;

    #[derive(Default)]
    pub struct MockStatusClaimService {
        index_map: DashMap<String, u32>,
    }

    impl StatusClaimService for MockStatusClaimService {
        type Error = Infallible;

        async fn obtain_status_claims(
            &self,
            attestation_type: &str,
            _batch_id: Uuid,
            base_url: BaseUrl,
            _expires: Option<DateTimeSeconds>,
            copies: NonZeroUsize,
        ) -> Result<VecNonEmpty<StatusClaim>, Self::Error> {
            let copies = copies.get() as u32;
            let url = base_url.join(attestation_type.replace(':', "-").as_str());
            let mut entry = self.index_map.entry(attestation_type.to_string()).or_insert(0);
            let start = *entry + 1;
            *entry += copies;
            let claims = (start..=*entry)
                .map(|idx| StatusClaim::StatusList(StatusListClaim { idx, uri: url.clone() }))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            Ok(claims)
        }
    }
}
