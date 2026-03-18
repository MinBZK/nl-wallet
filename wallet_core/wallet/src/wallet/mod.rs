mod attestations;
mod change_pin;
mod close_proximity_disclosure;
mod config;
mod disclosure;
mod disclosure_based_issuance;
mod history;
mod init;
mod instruction_client;
mod issuance;
mod lock;
mod notifications;
mod pin_recovery;
mod recovery_code;
mod registration;
mod reset;
mod revocation;
mod revocation_code;
mod state;
mod transfer;
mod uri;

#[cfg(test)]
mod test;

use std::marker::PhantomData;
use std::sync::Arc;

use cfg_if::cfg_if;
use parking_lot::Mutex;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::disclosure_session::VpDisclosureClient;
use openid4vc::issuance_session::HttpIssuanceSession;
use openid4vc::oidc::HttpOidcClient;
use openid4vc::oidc::OidcClient;
use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::close_proximity_disclosure::hardware::HardwareCloseProximityDisclosureClient;
use platform_support::hw_keystore::hardware::HardwareEncryptionKey;
use token_status_list::verification::reqwest::HttpStatusListClient;
use wallet_configuration::wallet_config::PidAttributesConfiguration;

use crate::account_provider::HttpAccountProviderClient;
use crate::config::WalletConfigurationRepository;
use crate::digid::DigidSessionState;
use crate::lock::WalletLock;
use crate::storage::DatabaseStorage;
use crate::storage::RegistrationData;
use crate::update_policy::UpdatePolicyRepository;
use crate::wallet::close_proximity_disclosure::CloseProximityDisclosureSession;
use crate::wallet::notifications::DirectNotificationsCallback;
use crate::wallet::pin_recovery::PinRecoverySession;

use self::attestations::AttestationsCallback;
use self::disclosure::WalletDisclosureSession;
use self::issuance::WalletIssuanceSession;

pub use self::close_proximity_disclosure::CloseProximityDisclosureUpdate;
pub use self::disclosure::AttributesNotAvailable;
pub use self::disclosure::DisclosureAttestationOptions;
pub use self::disclosure::DisclosureError;
pub use self::disclosure::DisclosureProposalPresentation;
pub use self::disclosure::DisclosureUriSource;
pub use self::disclosure_based_issuance::DisclosureBasedIssuanceError;
pub use self::history::HistoryError;
pub use self::history::RecentHistoryCallback;
pub use self::init::WalletClients;
pub use self::init::WalletInitError;
pub use self::issuance::IssuanceError;
pub use self::issuance::IssuanceResult;
pub use self::issuance::PidIssuancePurpose;
pub use self::lock::LockCallback;
pub use self::lock::UnlockMethod;
pub use self::lock::WalletUnlockError;
pub use self::notifications::ScheduledNotificationsCallback;
pub use self::pin_recovery::PinRecoveryError;
pub use self::recovery_code::RecoveryCodeError;
pub use self::registration::WalletRegistrationError;
pub use self::reset::ResetError;
pub use self::revocation_code::RevocationCodeError;
pub use self::state::BlockedReason;
pub use self::state::TransferRole;
pub use self::state::WalletState;
pub use self::transfer::TransferError;
pub use self::uri::UriIdentificationError;
pub use self::uri::UriType;

cfg_if! {
    if #[cfg(feature = "fake_attestation")] {
        type KeyHolderType = platform_support::attested_key::mock::PersistentMockAttestedKeyHolder;
    } else {
        type KeyHolderType = platform_support::attested_key::hardware::HardwareAttestedKeyHolder;
    }
}

#[derive(Debug, Default)]
enum WalletRegistration<A, G> {
    #[default]
    Unregistered,
    KeyIdentifierGenerated(String),
    Registered {
        attested_key: Arc<AttestedKey<A, G>>,
        data: RegistrationData,
    },
}

impl<A, G> WalletRegistration<A, G> {
    fn is_registered(&self) -> bool {
        match self {
            Self::Unregistered | Self::KeyIdentifierGenerated(_) => false,
            Self::Registered { .. } => true,
        }
    }

    fn as_key_and_registration_data(&self) -> Option<(&Arc<AttestedKey<A, G>>, &RegistrationData)> {
        match self {
            Self::Unregistered | Self::KeyIdentifierGenerated(_) => None,
            Self::Registered { attested_key, data } => Some((attested_key, data)),
        }
    }
}

#[derive(Debug)]
enum Session<OC: OidcClient, IS, DCS> {
    Digid {
        purpose: PidIssuancePurpose,
        session: DigidSessionState<OC>,
    },
    Issuance(WalletIssuanceSession<IS>),
    Disclosure(WalletDisclosureSession<DCS>),
    CloseProximityDisclosure(
        #[cfg_attr(
            not(test),
            expect(
                unused,
                reason = "will be used when continue_close_proximity_disclosure is implemented"
            )
        )]
        CloseProximityDisclosureSession,
    ),
    PinRecovery {
        pid_config: PidAttributesConfiguration,
        session: PinRecoverySession<OC, IS>,
    },
}

pub struct Wallet<
    CR = WalletConfigurationRepository,           // Repository<WalletConfiguration>
    UR = UpdatePolicyRepository,                  // Repository<VersionState>
    S = DatabaseStorage<HardwareEncryptionKey>,   // Storage
    AKH = KeyHolderType,                          // AttestedKeyHolder
    APC = HttpAccountProviderClient,              // AccountProviderClient
    OC = HttpOidcClient,                          // OidcClient for DigiD
    IS = HttpIssuanceSession,                     // IssuanceSession
    DCC = VpDisclosureClient,                     // DisclosureClient
    CPC = HardwareCloseProximityDisclosureClient, // CloseProximityDisclosureClient
    SLC = HttpStatusListClient,                   // StatusListClient,
> where
    AKH: AttestedKeyHolder,
    OC: OidcClient,
    DCC: DisclosureClient,
{
    config_repository: Arc<CR>,
    update_policy_repository: UR,
    storage: Arc<RwLock<S>>,
    key_holder: AKH,
    registration: WalletRegistration<AKH::AppleKey, AKH::GoogleKey>,
    account_provider_client: Arc<APC>,
    disclosure_client: DCC,
    close_proximity_disclosure: PhantomData<CPC>,
    status_list_client: Arc<SLC>,
    session: Option<Session<OC, IS, DCC::Session>>,
    lock: WalletLock,
    attestations_callback: Arc<Mutex<Option<AttestationsCallback>>>,
    recent_history_callback: Option<RecentHistoryCallback>,
    scheduled_notifications_callback: Arc<Mutex<Option<ScheduledNotificationsCallback>>>,
    direct_notifications_callback: Arc<Mutex<Option<DirectNotificationsCallback>>>,
    revocation_status_job_handle: Option<JoinHandle<()>>,
}
