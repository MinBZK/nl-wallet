use std::num::NonZeroUsize;

use uuid::Uuid;

use attestation_types::status_claim::StatusClaim;
use utils::date_time_seconds::DateTimeSeconds;
use utils::vec_at_least::VecNonEmpty;

#[trait_variant::make(Send)]
pub trait StatusListService {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn obtain_status_claims(
        &self,
        attestation_type: &str,
        batch_id: Uuid,
        expires: Option<DateTimeSeconds>,
        copies: NonZeroUsize,
    ) -> Result<VecNonEmpty<StatusClaim>, Self::Error>;
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::convert::Infallible;

    use dashmap::DashMap;

    use attestation_types::status_claim::StatusListClaim;
    use http_utils::urls::BaseUrl;

    use super::*;

    pub struct MockStatusListService {
        base_url: BaseUrl,
        index_map: DashMap<String, u32>,
    }

    impl MockStatusListService {
        pub fn from_base_url(base_url: BaseUrl) -> Self {
            Self {
                base_url,
                index_map: Default::default(),
            }
        }
    }

    impl Default for MockStatusListService {
        fn default() -> Self {
            Self {
                base_url: "https://example.com".parse().unwrap(),
                index_map: Default::default(),
            }
        }
    }

    impl StatusListService for MockStatusListService {
        type Error = Infallible;

        async fn obtain_status_claims(
            &self,
            attestation_type: &str,
            _batch_id: Uuid,
            _expires: Option<DateTimeSeconds>,
            copies: NonZeroUsize,
        ) -> Result<VecNonEmpty<StatusClaim>, Self::Error> {
            let copies = copies.get() as u32;
            let url = self.base_url.join(attestation_type.replace(':', "-").as_str());
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
