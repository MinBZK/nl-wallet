use base64::{engine::general_purpose::STANDARD, Engine};
use url::Url;

use platform_support::{
    hw_keystore::hardware::{HardwareEcdsaKey, HardwareEncryptionKey},
    utils::hardware::HardwareUtilities,
};
use wallet_common::account::jwt::EcdsaDecodingKey;

use crate::{
    account_server::RemoteAccountServerClient,
    config::{AccountServerConfiguration, Configuration, LocalConfigurationRepository, LockTimeoutConfiguration},
    storage::DatabaseStorage,
    wallet::WalletInitError,
};

pub type Wallet = crate::wallet::Wallet<
    LocalConfigurationRepository,
    RemoteAccountServerClient,
    DatabaseStorage<HardwareEncryptionKey, HardwareUtilities>,
    HardwareEcdsaKey,
>;

pub async fn init_wallet() -> Result<Wallet, WalletInitError> {
    // The initial configuration serves as the hardcoded fallback, for
    // when the app starts and no configuration from the Wallet Provider
    // is cached yet.
    let config = LocalConfigurationRepository::new_with_initial(|| Configuration {
        lock_timeouts: LockTimeoutConfiguration {
            inactive_timeout: 5 * 60,
            background_timeout: 5 * 60,
        },
        account_server: AccountServerConfiguration {
            base_url: Url::parse("http://localhost:3000/api/v1/").unwrap(),
            certificate_public_key: EcdsaDecodingKey::from_sec1(&STANDARD.decode("").unwrap()),
            instruction_result_public_key: EcdsaDecodingKey::from_sec1(&STANDARD.decode("").unwrap()),
        },
    });

    Wallet::new(config).await
}
