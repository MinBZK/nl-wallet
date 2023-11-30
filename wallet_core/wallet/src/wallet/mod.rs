mod config;
mod disclosure;
mod documents;
mod history;
mod init;
mod issuance;
mod lock;
mod registration;
mod uri;

#[cfg(test)]
mod tests;

use nl_wallet_mdoc::holder::{CborHttpClient, DisclosureSession};
use tokio::sync::RwLock;

use platform_support::hw_keystore::hardware::{HardwareEcdsaKey, HardwareEncryptionKey};

use crate::{
    account_provider::HttpAccountProviderClient,
    config::PreloadConfigurationRepository,
    digid::HttpDigidSession,
    lock::WalletLock,
    pid_issuer::HttpPidIssuerClient,
    storage::{DatabaseStorage, RegistrationData},
};

pub use self::{
    disclosure::{DisclosureError, DisclosureProposal},
    history::HistoryError,
    init::WalletInitError,
    issuance::PidIssuanceError,
    lock::WalletUnlockError,
    registration::WalletRegistrationError,
    uri::{UriIdentificationError, UriType},
};

use self::{config::ConfigurationCallback, documents::DocumentsCallback};

pub struct Wallet<
    CR = PreloadConfigurationRepository,        // ConfigurationRepository
    S = DatabaseStorage<HardwareEncryptionKey>, // Storage
    PEK = HardwareEcdsaKey,                     // PlatformEcdsaKey
    APC = HttpAccountProviderClient,            // AccountProviderClient
    DGS = HttpDigidSession,                     // DigidSession
    PIC = HttpPidIssuerClient,                  // PidIssuerClient
    MDS = DisclosureSession<CborHttpClient>,    // MdocDisclosureSession
> {
    config_repository: CR,
    storage: RwLock<S>,
    hw_privkey: PEK,
    account_provider_client: APC,
    digid_session: Option<DGS>,
    pid_issuer: PIC,
    disclosure_session: Option<MDS>,
    lock: WalletLock,
    registration: Option<RegistrationData>,
    config_callback: Option<ConfigurationCallback>,
    documents_callback: Option<DocumentsCallback>,
}
