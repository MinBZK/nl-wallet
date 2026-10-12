use std::time::Duration;

use attestation_types::status_claim::StatusClaim;
use attestation_types::status_claim::StatusListClaim;
use chrono::DateTime;
use chrono::Utc;
use crypto::EcdsaKey;
use crypto::server_keys::KeyPair;
use jwt::error::JwtSignError;
use token_status_list::status_list::StatusList;
use token_status_list::status_list::StatusType;
use token_status_list::status_list_token::StatusListToken;
use tokio::sync::Mutex;
use url::Url;
use utils::generator::Generator;

use crate::flags::WalletFlags;

/// If the cached [`StatusListToken`] held by [`KeyStorageStatusList`] expires within this period, it will be discarded
/// and [`KeyStorageStatusList`] will sign and cache a new one.
const EXPIRY_MARGIN: Duration = Duration::from_mins(5);

/// This type holds a cached [`SingleEntryStatusListToken`], which in turn contains a [`StatusListToken`] in which only
/// index 0 is used and indicates the revocation status of Wallet Provider's HSM. This is to be used as the Key Storage
/// type revocation status of an OpenID4VCI Key Attestation. Within the context of TS3, this refers to Option 1 of
/// section 2.5.2.
///
/// It also provides the [`StatusClaim`] to be included in Key Attestations as a convenience.
///
/// See: <https://github.com/eu-digital-identity-wallet/eudi-doc-standards-and-technical-specifications/blob/5924eb77ab4495d4dc0a874e54ac3e5de1fbd5b1/docs/technical-specifications/ts3-wallet-unit-attestation.md#252-ka-revocation>
#[derive(Debug)]
pub struct KeyStorageStatusList<K, F> {
    pub status_claim: StatusClaim,
    token_expiry: Duration,
    key_pair: KeyPair<K>,
    wallet_flags: F,
    cached_token: Mutex<Option<SingleEntryStatusListToken>>,
}

impl<K, F> KeyStorageStatusList<K, F> {
    pub fn new(public_uri: Url, token_expiry: Duration, key_pair: KeyPair<K>, wallet_flags: F) -> Self {
        let status_claim = StatusClaim::StatusList(StatusListClaim {
            idx: 0,
            uri: public_uri,
        });

        Self {
            status_claim,
            token_expiry,
            key_pair,
            wallet_flags,
            cached_token: Mutex::new(None),
        }
    }

    /// Either return the signed [`StatusListToken`] from cache or sign a new one, using the type's [`KeyPair`]. A new
    /// token is signed in the following situations:
    ///
    /// 1. It is the first time this method is called and the cache is empty.
    /// 2. The Wallet Solution's revocation status has changed since the cached value was signed.
    /// 2. The cached token is about to expire.
    pub async fn status_list_token(&self, time: &impl Generator<DateTime<Utc>>) -> Result<StatusListToken, JwtSignError>
    where
        K: EcdsaKey,
        F: WalletFlags,
    {
        let mut cached_token = self.cached_token.lock().await;

        let status = if self.wallet_flags.solution_is_revoked() {
            StatusType::Invalid
        } else {
            StatusType::Valid
        };
        let now = time.generate();

        let token = match cached_token
            .as_ref()
            .filter(|single_token| single_token.status == status && now + EXPIRY_MARGIN < single_token.expires)
        {
            Some(single_token) => single_token.token.clone(),
            None => {
                let StatusClaim::StatusList(StatusListClaim { uri, .. }) = &self.status_claim;
                let single_token =
                    SingleEntryStatusListToken::try_new(uri.clone(), status, now + self.token_expiry, &self.key_pair)
                        .await?;

                let token = single_token.token.clone();
                cached_token.replace(single_token);

                token
            }
        };

        Ok(token)
    }
}

/// Wraps a [`StatusListToken`] that holds just a single value at index 0, along with that [`StatusType`] stored at that
/// index and the time at which the token expires.
#[derive(Debug)]
struct SingleEntryStatusListToken {
    token: StatusListToken,
    status: StatusType,
    expires: DateTime<Utc>,
}

impl SingleEntryStatusListToken {
    pub async fn try_new(
        uri: Url,
        status: StatusType,
        expires: DateTime<Utc>,
        key_pair: &KeyPair<impl EcdsaKey>,
    ) -> Result<Self, JwtSignError> {
        let mut list = StatusList::new(1);
        list.insert(0, status);

        let token = StatusListToken::builder(uri, list.pack())
            .exp(Some(expires))
            .sign(key_pair)
            .await?;

        Ok(Self { token, status, expires })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;
    use std::time::Duration;

    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use crypto::x509::BorrowingCertificate;
    use futures::FutureExt;
    use token_status_list::status_list::StatusType;
    use token_status_list::status_list_token::StatusListToken;
    use url::Url;
    use utils::generator::Generator;
    use utils::generator::mock::MockTimeGenerator;

    use super::KeyStorageStatusList;
    use super::SingleEntryStatusListToken;
    use crate::flags::WalletFlags;
    use crate::flags::mock::StubWalletFlags;

    static TSL_ANCHORS_AND_KEY_PAIR: LazyLock<(TrustAnchors, KeyPair)> = LazyLock::new(|| {
        let ca = Ca::generate_wptl_mock_ca().unwrap();
        let trust_anchors = TrustAnchors::from(&ca);
        let key_pair = ca.generate_key_storage_status_list_mock().unwrap();

        (trust_anchors, key_pair)
    });

    #[test]
    fn test_single_entry_status_list_token() {
        let (trust_anchors, key_pair) = LazyLock::force(&TSL_ANCHORS_AND_KEY_PAIR);
        let time = MockTimeGenerator::default();

        let uri = "https://example.com/tsl".parse::<Url>().unwrap();
        let expires = time.generate() + Duration::from_hours(1);
        let status = StatusType::Invalid;

        let single_token = SingleEntryStatusListToken::try_new(uri.clone(), status, expires, key_pair)
            .now_or_never()
            .unwrap()
            .expect("signing single entry token status list should succeed");

        let claims = single_token
            .token
            .parse_and_verify(
                trust_anchors,
                key_pair.certificate().to_canonical_distinguished_name().unwrap(),
                uri,
                &time,
            )
            .expect("verifying single entry token status list should succeed");

        // Only the first index in the status list should be set. Anything beyond the first index should result in a
        // default status of `StatusType::Valid`.
        let status_list = claims.status_list.unpack();
        assert_eq!(status_list.get(0), StatusType::Invalid);
        assert_eq!(status_list.get(1), StatusType::Valid);
    }

    /// Helper function to extract the one [`StatusType`] from a [`StatusListToken`] contained
    /// [`SingleEntryStatusListToken`].
    fn key_storage_status(
        token: &StatusListToken,
        trust_anchors: &TrustAnchors,
        certificate: &BorrowingCertificate,
        public_uri: Url,
        time: &MockTimeGenerator,
    ) -> StatusType {
        token
            .parse_and_verify(
                trust_anchors,
                certificate.to_canonical_distinguished_name().unwrap(),
                public_uri,
                time,
            )
            .expect("verifying single entry token status list should succeed")
            .status_list
            .unpack()
            .get(0)
    }

    #[test]
    fn test_key_storage_status_list() {
        let (trust_anchors, key_pair) = LazyLock::force(&TSL_ANCHORS_AND_KEY_PAIR);
        let wallet_flags = StubWalletFlags::default();
        let time = MockTimeGenerator::default();

        let public_uri = "https://example.com/tsl".parse::<Url>().unwrap();
        let token_expiry = Duration::from_hours(1);

        let solution_status_list =
            KeyStorageStatusList::new(public_uri.clone(), token_expiry, key_pair.clone(), wallet_flags.clone());

        // Fetching the `StatusListToken` initially should result in a valid status, as the Wallet Solution has not
        // been revoked.
        let first_token = solution_status_list
            .status_list_token(&time)
            .now_or_never()
            .unwrap()
            .expect("retrieving token status list from solution revoked status list should succeed");

        assert_eq!(
            key_storage_status(
                &first_token,
                trust_anchors,
                key_pair.certificate(),
                public_uri.clone(),
                &time
            ),
            StatusType::Valid
        );

        // Fetching the `StatusListToken` after 10 minutes should result in the same one being returned from cache, as
        // this fits well within the expiration period of 1 hour.
        *time.time.write() += Duration::from_mins(10);

        let second_token = solution_status_list
            .status_list_token(&time)
            .now_or_never()
            .unwrap()
            .expect("retrieving token status list from solution revoked status list should succeed");

        assert_eq!(
            first_token.as_ref().serialization(),
            second_token.as_ref().serialization()
        );

        // Fetching the `StatusListToken` after 55 minutes should result in a new one being signed, as this should just
        // hit the margin at which the cache is invalidated, i.e. an hour minutes 5 minutes. The status should still be
        // valid.
        *time.time.write() += Duration::from_mins(45);

        let third_token = solution_status_list
            .status_list_token(&time)
            .now_or_never()
            .unwrap()
            .expect("retrieving token status list from solution revoked status list should succeed");

        assert_ne!(
            first_token.as_ref().serialization(),
            third_token.as_ref().serialization()
        );

        assert_eq!(
            key_storage_status(
                &third_token,
                trust_anchors,
                key_pair.certificate(),
                public_uri.clone(),
                &time
            ),
            StatusType::Valid
        );

        // After revoking the solution, a `StatusListToken` with an invalid status should be returned instead.
        wallet_flags.set_solution_revoked().now_or_never().unwrap().unwrap();

        let fourth_token = solution_status_list
            .status_list_token(&time)
            .now_or_never()
            .unwrap()
            .expect("retrieving token status list from solution revoked status list should succeed");

        assert_eq!(
            key_storage_status(
                &fourth_token,
                trust_anchors,
                key_pair.certificate(),
                public_uri.clone(),
                &time
            ),
            StatusType::Invalid
        );
    }
}
