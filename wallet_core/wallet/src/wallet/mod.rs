mod config;
mod disclosure;
mod documents;
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
    config::LocalConfigurationRepository,
    digid::HttpDigidSession,
    lock::WalletLock,
    pid_issuer::HttpPidIssuerClient,
    storage::{DatabaseStorage, RegistrationData},
};

pub use self::{
    disclosure::{DisclosureError, DisclosureProposal},
    init::WalletInitError,
    issuance::PidIssuanceError,
    lock::WalletUnlockError,
    registration::WalletRegistrationError,
    uri::{UriIdentificationError, UriType},
};

use self::{config::ConfigurationCallback, documents::DocumentsCallback};

pub struct Wallet<
    C = LocalConfigurationRepository,
    S = DatabaseStorage<HardwareEncryptionKey>,
    K = HardwareEcdsaKey,
    A = HttpAccountProviderClient,
    D = HttpDigidSession,
    P = HttpPidIssuerClient,
    R = DisclosureSession<CborHttpClient>,
> {
    config_repository: C,
    storage: RwLock<S>,
    hw_privkey: K,
    account_provider_client: A,
    digid_session: Option<D>,
    pid_issuer: P,
    disclosure_session: Option<R>,
    lock: WalletLock,
    registration: Option<RegistrationData>,
    config_callback: Option<ConfigurationCallback>,
    documents_callback: Option<DocumentsCallback>,
}
