mod change_pin;
mod config;
mod disclosure;
mod documents;
mod history;
mod init;
mod instruction_client;
mod issuance;
mod lock;
mod registration;
mod reset;
mod uri;

#[cfg(test)]
mod test;

use cfg_if::cfg_if;
use tokio::sync::RwLock;
use uuid::Uuid;

use openid4vc::disclosure_session::DisclosureSession;
use openid4vc::disclosure_session::HttpVpMessageClient;
use openid4vc::issuance_session::HttpIssuanceSession;
use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::hw_keystore::hardware::HardwareEncryptionKey;

use crate::account_provider::HttpAccountProviderClient;
use crate::config::WalletConfigurationRepository;
use crate::issuance::HttpDigidSession;
use crate::lock::WalletLock;
use crate::storage::DatabaseStorage;
use crate::storage::RegistrationData;
use crate::update_policy::UpdatePolicyRepository;
use crate::wte::WpWteIssuanceClient;

pub use self::disclosure::DisclosureError;
pub use self::disclosure::DisclosureProposal;
pub use self::documents::DocumentsCallback;
pub use self::history::EventConversionError;
pub use self::history::EventStatus;
pub use self::history::EventStorageError;
pub use self::history::HistoryError;
pub use self::history::HistoryEvent;
pub use self::history::RecentHistoryCallback;
pub use self::init::WalletInitError;
pub use self::issuance::PidIssuanceError;
pub use self::lock::LockCallback;
pub use self::lock::UnlockMethod;
pub use self::lock::WalletUnlockError;
pub use self::registration::WalletRegistrationError;
pub use self::reset::ResetError;
pub use self::uri::UriIdentificationError;
pub use self::uri::UriType;

use self::issuance::PidIssuanceSession;

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
        attested_key: AttestedKey<A, G>,
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

    fn as_key_and_registration_data(&self) -> Option<(&AttestedKey<A, G>, &RegistrationData)> {
        match self {
            Self::Unregistered | Self::KeyIdentifierGenerated(_) => None,
            Self::Registered { attested_key, data } => Some((attested_key, data)),
        }
    }
}

pub struct Wallet<
    CR = WalletConfigurationRepository,                 // Repository<WalletConfiguration>
    UR = UpdatePolicyRepository,                        // Repository<VersionState>
    S = DatabaseStorage<HardwareEncryptionKey>,         // Storage
    AKH = KeyHolderType,                                // AttestedKeyHolder
    APC = HttpAccountProviderClient,                    // AccountProviderClient
    DS = HttpDigidSession,                              // DigidSession
    IS = HttpIssuanceSession,                           // IssuanceSession
    MDS = DisclosureSession<HttpVpMessageClient, Uuid>, // MdocDisclosureSession
    WIC = WpWteIssuanceClient,                          // WteIssuanceClient
> where
    AKH: AttestedKeyHolder,
{
    config_repository: CR,
    update_policy_repository: UR,
    storage: RwLock<S>,
    key_holder: AKH,
    registration: WalletRegistration<AKH::AppleKey, AKH::GoogleKey>,
    account_provider_client: APC,
    issuance_session: Option<PidIssuanceSession<DS, IS>>,
    disclosure_session: Option<MDS>,
    wte_issuance_client: WIC,
    lock: WalletLock,
    documents_callback: Option<DocumentsCallback>,
    recent_history_callback: Option<RecentHistoryCallback>,
}
