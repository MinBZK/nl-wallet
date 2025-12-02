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
    Blocked { reason: BlockedReason },
    Unregistered,
    Locked { sub_state: Box<WalletState> },
    // The following variants may appear in `Locked { sub_state }`
    Empty,
    TransferPossible,
    Transferring { role: TransferRole },
    InDisclosureFlow,
    InIssuanceFlow,
    InPinChangeFlow,
    InPinRecoveryFlow,
    Ready,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlockedReason {
    RequiresAppUpdate,
    BlockedByWalletProvider,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TransferRole {
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
            // TODO
            return Ok(WalletState::Blocked {
                reason: BlockedReason::RequiresAppUpdate,
            });
        }

        if !self.has_registration() {
            return Ok(WalletState::Unregistered);
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

        if !read_storage.has_any_attestations().await? {
            return Ok(WalletState::Empty);
        }

        if let Some(transfer_data) = read_storage.fetch_data::<TransferData>().await? {
            return Ok(transfer_data
                .key_data
                .map(|key_data| {
                    let role = match key_data {
                        TransferKeyData::Source { .. } => TransferRole::Source,
                        TransferKeyData::Destination { .. } => TransferRole::Destination,
                    };
                    WalletState::Transferring { role }
                })
                .unwrap_or(WalletState::TransferPossible));
        }

        if let Some(session) = &self.session {
            return match session {
                Session::Digid { .. } => Ok(WalletState::InIssuanceFlow),
                Session::Issuance(_) => Ok(WalletState::InIssuanceFlow),
                Session::Disclosure(_) => Ok(WalletState::InDisclosureFlow),
                Session::PinRecovery { .. } => Ok(WalletState::InPinRecoveryFlow),
            };
        }
        if self.storage.get_change_pin_state().await?.is_some() {
            return Ok(WalletState::InPinChangeFlow);
        }

        Ok(WalletState::Ready)
    }
}

#[cfg(test)]
mod tests {
    use futures::FutureExt;
    use http_utils::tls::pinning::TlsPinningConfig;
    use josekit::jwk::Jwk;
    use josekit::jwk::alg::ec::EcCurve;
    use josekit::jwk::alg::ec::EcKeyPair;
    use openid4vc::mock::MockIssuanceSession;
    use rstest::rstest;
    use uuid::Uuid;

    use attestation_data::disclosure_type::DisclosureType;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use openid4vc::disclosure_session::mock::MockDisclosureSession;
    use openid4vc::issuance_session::IssuedCredential;
    use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;

    use crate::PidIssuancePurpose;
    use crate::TransferRole;
    use crate::WalletState;
    use crate::pin::change::State;
    use crate::repository::Repository;
    use crate::storage::ChangePinData;
    use crate::storage::TransferData;
    use crate::storage::TransferKeyData;
    use crate::test::MockDigidSession;
    use crate::wallet::PinRecoverySession;
    use crate::wallet::Session;
    use crate::wallet::WalletDisclosureSession;
    use crate::wallet::disclosure::RedirectUriPurpose;
    use crate::wallet::issuance::WalletIssuanceSession;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::WalletDeviceVendor;
    use crate::wallet::test::create_example_pid_sd_jwt;
    use crate::wallet::test::mock_issuance_session;

    impl WalletState {
        fn lock(self) -> Self {
            Self::Locked {
                sub_state: Box::new(self),
            }
        }
    }

    #[tokio::test]
    async fn test_unregistered_wallet() {
        let wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;
        let wallet_state = wallet.get_state().await.unwrap();
        assert_eq!(wallet_state, WalletState::Unregistered);
    }

    #[tokio::test]
    async fn test_init_registation_wallet() {
        let wallet = TestWalletMockStorage::new_init_registration(WalletDeviceVendor::Apple)
            .await
            .unwrap();
        let wallet_state = wallet.get_state().await.unwrap();
        assert_eq!(wallet_state, WalletState::Unregistered);
    }

    #[rstest]
    #[case::empty(false, false, WalletState::Empty)]
    #[case::registered_and_unlocked(false, true, WalletState::Ready)]
    #[case::registered_and_locked(true, true, WalletState::Ready.lock())]
    #[tokio::test]
    async fn test_empty_and_registered_wallet(
        #[case] is_locked: bool,
        #[case] has_attestations: bool,
        #[case] expected_state: WalletState,
    ) {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        if is_locked {
            wallet.lock();
        }

        let storage = wallet.mut_storage();
        storage
            .expect_has_any_attestations()
            .return_once(move || Ok(has_attestations));
        storage.expect_fetch_data::<ChangePinData>().return_once(|| Ok(None));
        storage.expect_fetch_data::<TransferData>().return_once(|| Ok(None));

        let wallet_state = wallet.get_state().await.unwrap();
        assert_eq!(wallet_state, expected_state);
    }

    #[rstest]
    #[case::pin_state_begin(false, State::Begin, WalletState::InPinChangeFlow)]
    #[case::pin_state_commit(false, State::Commit, WalletState::InPinChangeFlow)]
    #[case::pin_state_rollback(false, State::Rollback, WalletState::InPinChangeFlow)]
    #[case::locked_pin_state(true, ChangePinData {state: State::Begin}, WalletState::InPinChangeFlow.lock())]
    #[tokio::test]
    async fn test_change_pin_data(
        #[case] is_locked: bool,
        #[case] change_pin_data: impl Into<ChangePinData>,
        #[case] expected_state: WalletState,
    ) {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        if is_locked {
            wallet.lock();
        }

        let storage = wallet.mut_storage();
        storage.expect_has_any_attestations().return_once(|| Ok(true));
        let change_pin_data = change_pin_data.into();
        storage
            .expect_fetch_data::<ChangePinData>()
            .return_once(move || Ok(Some(change_pin_data)));

        storage.expect_fetch_data::<TransferData>().return_once(|| Ok(None));

        let wallet_state = wallet.get_state().await.unwrap();
        assert_eq!(wallet_state, expected_state);
    }

    #[rstest]
    #[case::transfer_possible(false, empty_transfer_data(), WalletState::TransferPossible)]
    #[case::transferring_source(false, source_transfer_data(), WalletState::Transferring { role: TransferRole::Source })]
    #[case::transferring_destination(false, destination_transfer_data(), WalletState::Transferring { role: TransferRole::Destination })]
    #[case::locked_transfer_possible(true, empty_transfer_data(), WalletState::TransferPossible.lock())]
    #[tokio::test]
    async fn test_wallet_transfer(
        #[case] is_locked: bool,
        #[case] transfer_data: TransferData,
        #[case] expected_state: WalletState,
    ) {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        if is_locked {
            wallet.lock();
        }

        let storage = wallet.mut_storage();
        storage.expect_has_any_attestations().return_once(|| Ok(true));
        storage.expect_fetch_data::<ChangePinData>().return_once(|| Ok(None));
        storage
            .expect_fetch_data::<TransferData>()
            .return_once(move || Ok(Some(transfer_data)));

        let wallet_state = wallet.get_state().await.unwrap();
        assert_eq!(wallet_state, expected_state);
    }

    #[rstest]
    #[case::digid(false, digid_session(), WalletState::InIssuanceFlow)]
    #[case::issuance(false, issuance_session(), WalletState::InIssuanceFlow)]
    #[case::disclosure(false, disclosure_session(), WalletState::InDisclosureFlow)]
    #[case::pin_recovery(false, pin_recovery_session(), WalletState::InPinRecoveryFlow)]
    #[case::locked_digid(true, digid_session(), WalletState::InIssuanceFlow.lock())]
    #[tokio::test]
    async fn test_wallet_session(
        #[case] is_locked: bool,
        #[case] session: Session<MockDigidSession<TlsPinningConfig>, MockIssuanceSession, MockDisclosureSession>,
        #[case] expected_state: WalletState,
    ) {
        // Prepare a registered and unlocked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        if is_locked {
            wallet.lock();
        }

        wallet.session = Some(session);

        let storage = wallet.mut_storage();
        storage.expect_has_any_attestations().return_once(|| Ok(true));
        storage.expect_fetch_data::<ChangePinData>().return_once(|| Ok(None));
        storage.expect_fetch_data::<TransferData>().return_once(|| Ok(None));

        let wallet_state = wallet.get_state().await.unwrap();

        assert_eq!(wallet_state, expected_state);
    }

    #[rstest]
    #[case::check_locked(
        true,
        true,
        Some(State::Begin),
        Some(empty_transfer_data()),
        Some(digid_session()),
        WalletState::TransferPossible.lock(),
    )]
    #[case::check_transfer_first(
        false,
        true,
        Some(State::Begin),
        Some(empty_transfer_data()),
        Some(digid_session()),
        WalletState::TransferPossible
    )]
    #[case::check_session_second(
        false,
        true,
        Some(State::Begin),
        None,
        Some(digid_session()),
        WalletState::InIssuanceFlow
    )]
    #[case::check_pin_state_third(false, true, Some(State::Begin), None, None, WalletState::InPinChangeFlow)]
    #[case::check_attestations_fourth(false, true, None::<State>, None, None, WalletState::Ready)]
    #[case(false, false, None::<State>, None, None, WalletState::Empty)]
    #[tokio::test]
    async fn test_precedence_of_checks(
        #[case] is_locked: bool,
        #[case] has_attestations: bool,
        #[case] change_pin_data: Option<impl Into<ChangePinData>>,
        #[case] transfer_data: Option<TransferData>,
        #[case] session: Option<
            Session<MockDigidSession<TlsPinningConfig>, MockIssuanceSession, MockDisclosureSession>,
        >,
        #[case] expected_state: WalletState,
    ) {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        if is_locked {
            wallet.lock();
        }

        wallet.session = session;

        let storage = wallet.mut_storage();
        storage
            .expect_has_any_attestations()
            .return_once(move || Ok(has_attestations));
        let change_pin_data = change_pin_data.map(Into::into);
        storage
            .expect_fetch_data::<ChangePinData>()
            .return_once(move || Ok(change_pin_data));
        storage
            .expect_fetch_data::<TransferData>()
            .return_once(move || Ok(transfer_data.clone()));

        let wallet_state = wallet.get_state().await.unwrap();
        assert_eq!(wallet_state, expected_state);
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

    fn digid_session() -> Session<MockDigidSession<TlsPinningConfig>, MockIssuanceSession, MockDisclosureSession> {
        Session::Digid {
            purpose: PidIssuancePurpose::Enrollment,
            session: MockDigidSession::new(),
        }
    }

    fn issuance_session() -> Session<MockDigidSession<TlsPinningConfig>, MockIssuanceSession, MockDisclosureSession> {
        // Create a mock OpenID4VCI session that accepts the PID with a single
        // instance of `MdocCopies`, which contains a single valid `Mdoc`.
        let (sd_jwt, _metadata) = create_example_pid_sd_jwt();
        let (pid_issuer, attestations) = mock_issuance_session(
            IssuedCredential::SdJwt {
                key_identifier: "key_id".to_string(),
                sd_jwt: sd_jwt.clone(),
            },
            String::from(PID_ATTESTATION_TYPE),
            VerifiedTypeMetadataDocuments::nl_pid_example(),
        );
        Session::Issuance(WalletIssuanceSession::new(
            Some(PidIssuancePurpose::Enrollment),
            attestations,
            pid_issuer,
        ))
    }

    fn disclosure_session() -> Session<MockDigidSession<TlsPinningConfig>, MockIssuanceSession, MockDisclosureSession> {
        Session::Disclosure(WalletDisclosureSession::new_missing_attributes(
            RedirectUriPurpose::Browser,
            DisclosureType::Regular,
            MockDisclosureSession::new(),
        ))
    }

    fn pin_recovery_session() -> Session<MockDigidSession<TlsPinningConfig>, MockIssuanceSession, MockDisclosureSession>
    {
        let wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple)
            .now_or_never()
            .unwrap();

        Session::PinRecovery {
            pid_config: wallet.config_repository.get().pid_attributes.clone(),
            session: PinRecoverySession::Digid(MockDigidSession::new()),
        }
    }
}
