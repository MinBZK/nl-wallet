mod attestations;
mod change_pin;
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
mod state;
mod transfer;
mod uri;

#[cfg(test)]
mod test;

use std::sync::Arc;

use cfg_if::cfg_if;
use parking_lot::Mutex;
use tokio::sync::RwLock;
use tokio::task::AbortHandle;

use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::disclosure_session::VpDisclosureClient;
use openid4vc::issuance_session::HttpIssuanceSession;
use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::hw_keystore::hardware::HardwareEncryptionKey;
use token_status_list::verification::reqwest::HttpStatusListClient;
use wallet_configuration::wallet_config::PidAttributesConfiguration;

use crate::account_provider::HttpAccountProviderClient;
use crate::config::WalletConfigurationRepository;
use crate::digid::DigidClient;
use crate::digid::HttpDigidClient;
use crate::lock::WalletLock;
use crate::storage::DatabaseStorage;
use crate::storage::RegistrationData;
use crate::update_policy::UpdatePolicyRepository;
use crate::wallet::pin_recovery::PinRecoverySession;

use self::attestations::AttestationsCallback;
use self::disclosure::WalletDisclosureSession;
use self::issuance::WalletIssuanceSession;

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
pub use self::notifications::NotificationsCallback;
pub use self::pin_recovery::PinRecoveryError;
pub use self::recovery_code::RecoveryCodeError;
pub use self::registration::WalletRegistrationError;
pub use self::reset::ResetError;
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
enum Session<DS, IS, DCS> {
    Digid {
        purpose: PidIssuancePurpose,
        session: DS,
    },
    Issuance(WalletIssuanceSession<IS>),
    Disclosure(WalletDisclosureSession<DCS>),
    PinRecovery {
        pid_config: PidAttributesConfiguration,
        session: PinRecoverySession<DS, IS>,
    },
}

pub struct Wallet<
    CR = WalletConfigurationRepository,         // Repository<WalletConfiguration>
    UR = UpdatePolicyRepository,                // Repository<VersionState>
    S = DatabaseStorage<HardwareEncryptionKey>, // Storage
    AKH = KeyHolderType,                        // AttestedKeyHolder
    APC = HttpAccountProviderClient,            // AccountProviderClient
    DC = HttpDigidClient,                       // DigidClient
    IS = HttpIssuanceSession,                   // IssuanceSession
    DCC = VpDisclosureClient,                   // DisclosureClient
    SLC = HttpStatusListClient,                 // StatusListClient,
> where
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    DCC: DisclosureClient,
{
    config_repository: Arc<CR>,
    update_policy_repository: UR,
    storage: Arc<RwLock<S>>,
    key_holder: AKH,
    registration: WalletRegistration<AKH::AppleKey, AKH::GoogleKey>,
    account_provider_client: Arc<APC>,
    digid_client: DC,
    disclosure_client: DCC,
    status_list_client: Arc<SLC>,
    session: Option<Session<DC::Session, IS, DCC::Session>>,
    lock: WalletLock,
    attestations_callback: Arc<Mutex<Option<AttestationsCallback>>>,
    recent_history_callback: Option<RecentHistoryCallback>,
    notifications_callback: Option<NotificationsCallback>,
    revocation_status_job_handle: Option<AbortHandle>,
}
