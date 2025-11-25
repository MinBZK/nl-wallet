use tracing::instrument;

use error_category::ErrorCategory;
use openid4vc::disclosure_session::DisclosureClient;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;

use crate::Wallet;
use crate::digid::DigidClient;
use crate::errors::StorageError;
use crate::pin::change::ChangePinStorage;
use crate::repository::Repository;
use crate::storage::Storage;
use crate::storage::TransferData;
use crate::storage::TransferKeyData;
use crate::wallet::Session;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum WalletStateError {
    #[error("error fetching data from storage: {0}")]
    Storage(#[from] StorageError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum WalletState {
    WalletBlocked { reason: WalletBlockedReason },
    Registration,
    Empty,
    Locked { sub_state: Box<WalletState> },
    TransferPossible,
    Transferring { role: WalletTransferRole },
    Disclosure,
    Issuance,
    PinChange,
    PinRecovery,
    Ready,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WalletBlockedReason {
    RequiresAppUpdate,
    BlockedByWalletProvider,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WalletTransferRole {
    Source,
    Destination,
}

impl<CR, UR, S, AKH, APC, DC, IS, DCC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC>
where
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    DCC: DisclosureClient,
{
    #[instrument(skip_all)]
    pub async fn get_state(&self) -> Result<WalletState, WalletStateError> {
        if self.is_blocked() {
            return Ok(WalletState::WalletBlocked {
                reason: WalletBlockedReason::RequiresAppUpdate,
            });
        }

        if !self.has_registration() {
            return Ok(WalletState::Registration);
        }

        let flow_state = self.get_flow_state().await?;

        if self.is_locked() {
            Ok(WalletState::Locked {
                sub_state: Box::new(flow_state),
            })
        } else {
            Ok(flow_state)
        }
    }

    async fn get_flow_state(&self) -> Result<WalletState, WalletStateError> {
        let read_storage = self.storage.read().await;

        if read_storage.fetch_unique_attestations().await?.is_empty() {
            return Ok(WalletState::Empty);
        }

        if let Some(transfer_data) = read_storage.fetch_data::<TransferData>().await? {
            return Ok(transfer_data
                .key_data
                .map(|key_data| {
                    let role = match key_data {
                        TransferKeyData::Source { .. } => WalletTransferRole::Source,
                        TransferKeyData::Destination { .. } => WalletTransferRole::Destination,
                    };
                    WalletState::Transferring { role }
                })
                .unwrap_or(WalletState::TransferPossible));
        }

        if let Some(session) = &self.session {
            return match session {
                Session::Digid(_) => Ok(WalletState::Issuance),
                Session::Issuance(_) => Ok(WalletState::Issuance),
                Session::Disclosure(_) => Ok(WalletState::Disclosure),
                Session::PinRecovery { .. } => Ok(WalletState::PinRecovery),
            };
        }
        if self.storage.get_change_pin_state().await?.is_some() {
            return Ok(WalletState::PinChange);
        }

        Ok(WalletState::Ready)
    }
}

#[cfg(test)]
mod tests {
    use josekit::jwk::Jwk;
    use josekit::jwk::alg::ec::EcCurve;
    use josekit::jwk::alg::ec::EcKeyPair;
    use rstest::rstest;
    use uuid::Uuid;

    use crate::WalletState;
    use crate::WalletTransferRole;
    use crate::pin::change::State;
    use crate::storage::ChangePinData;
    use crate::storage::StoredAttestation;
    use crate::storage::StoredAttestationCopy;
    use crate::storage::TransferData;
    use crate::storage::TransferKeyData;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::WalletDeviceVendor;
    use crate::wallet::test::create_example_pid_sd_jwt;

    #[tokio::test]
    async fn test_unregistered_wallet() {
        let wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;
        let wallet_state = wallet.get_state().await.unwrap();
        assert_eq!(wallet_state, WalletState::Registration);
    }

    #[tokio::test]
    async fn test_init_registation_wallet() {
        let wallet = TestWalletMockStorage::new_init_registration(WalletDeviceVendor::Apple)
            .await
            .unwrap();
        let wallet_state = wallet.get_state().await.unwrap();
        assert_eq!(wallet_state, WalletState::Registration);
    }

    #[rstest]
    #[case(vec![], None, None, WalletState::Empty)]
    #[case(vec![], None, Some(empty_transfer_data()), WalletState::Empty)]
    #[case(vec![], None, Some(source_transfer_data()), WalletState::Empty)]
    #[case(vec![], None, Some(destination_transfer_data()), WalletState::Empty)]
    #[case(vec![], Some(ChangePinData {state: State::Begin}), None, WalletState::Empty)]
    #[case(vec![], Some(ChangePinData {state: State::Commit}), None, WalletState::Empty)]
    #[case(vec![], Some(ChangePinData {state: State::Rollback}), None, WalletState::Empty)]
    #[case(vec![], Some(ChangePinData {state: State::Commit}), Some(source_transfer_data()), WalletState::Empty)]
    #[case(stored_attestation(), None, None, WalletState::Ready)]
    #[case(
        stored_attestation(),
        None,
        Some(empty_transfer_data()),
        WalletState::TransferPossible
    )]
    #[case(stored_attestation(), None, Some(source_transfer_data()), WalletState::Transferring { role: WalletTransferRole::Source })]
    #[case(stored_attestation(), None, Some(destination_transfer_data()), WalletState::Transferring { role: WalletTransferRole::Destination })]
    #[case(stored_attestation(), Some(ChangePinData {state: State::Begin}), None, WalletState::PinChange)]
    #[case(stored_attestation(), Some(ChangePinData {state: State::Commit}), None, WalletState::PinChange)]
    #[case(stored_attestation(), Some(ChangePinData {state: State::Rollback}), None, WalletState::PinChange)]
    #[case(stored_attestation(), Some(ChangePinData {state: State::Commit}), Some(source_transfer_data()), WalletState::Transferring { role: WalletTransferRole::Source })]
    #[tokio::test]
    async fn test_unregistered_and_unlocked_wallet(
        #[case] stored_attestations: Vec<StoredAttestationCopy>,
        #[case] change_pin_data: Option<ChangePinData>,
        #[case] transfer_data: Option<TransferData>,
        #[case] expected_state: WalletState,
    ) {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let storage = wallet.mut_storage();
        storage
            .expect_fetch_unique_attestations()
            .return_once(move || Ok(stored_attestations));
        storage
            .expect_fetch_data::<ChangePinData>()
            .return_once(move || Ok(change_pin_data.clone()));
        storage
            .expect_fetch_data::<TransferData>()
            .return_once(move || Ok(transfer_data.clone()));

        let wallet_state = wallet.get_state().await.unwrap();
        assert_eq!(wallet_state, expected_state);
    }

    fn stored_attestation() -> Vec<StoredAttestationCopy> {
        let (sd_jwt, sd_jwt_vc_metadata) = create_example_pid_sd_jwt();

        vec![StoredAttestationCopy::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            StoredAttestation::SdJwt {
                key_identifier: "sd_jwt_key_id".to_string(),
                sd_jwt,
            },
            sd_jwt_vc_metadata,
        )]
    }

    fn some_jwk() -> Jwk {
        let key_pair = EcKeyPair::generate(EcCurve::P256).unwrap();
        key_pair.to_jwk_public_key()
    }

    fn source_transfer_data() -> TransferData {
        TransferData {
            transfer_session_id: Uuid::new_v4().into(),
            key_data: Some(TransferKeyData::Source { public_key: some_jwk() }),
        }
    }

    fn destination_transfer_data() -> TransferData {
        TransferData {
            transfer_session_id: Uuid::new_v4().into(),
            key_data: Some(TransferKeyData::Destination {
                private_key: some_jwk(),
            }),
        }
    }

    fn empty_transfer_data() -> TransferData {
        TransferData {
            transfer_session_id: Uuid::new_v4().into(),
            key_data: None,
        }
    }
}
