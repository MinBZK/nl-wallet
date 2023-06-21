use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Ok, Result};
use flutter_rust_bridge::StreamSink;
use tokio::sync::{OnceCell, RwLock};

use macros::async_runtime;
use wallet::{init_wallet, validate_pin, Wallet};

use crate::{
    async_runtime::init_async_runtime,
    logging::init_logging,
    models::{
        config::FlutterConfiguration,
        pin::PinValidationResult,
        uri_flow_event::{DigidState, UriFlowEvent},
        wallet::WalletUnlockResult,
    },
    stream::ClosingStreamSink,
};

struct WalletApiEnvironment {
    wallet: RwLock<Wallet>,
}

impl WalletApiEnvironment {
    fn new(wallet: Wallet) -> Self {
        WalletApiEnvironment {
            wallet: RwLock::new(wallet),
        }
    }
}

static WALLET_API_ENVIRONMENT: OnceCell<WalletApiEnvironment> = OnceCell::const_new();

fn wallet_environment() -> &'static WalletApiEnvironment {
    WALLET_API_ENVIRONMENT
        .get()
        .expect("Wallet must be initialized. Please execute `init()` first.")
}

fn wallet() -> &'static RwLock<Wallet> {
    &wallet_environment().wallet
}

pub fn init() -> Result<()> {
    // Initialize platform specific logging and set the log level.
    // As creating the wallet below could fail and init() could be called again,
    // init_logging() should not fail when being called more than once.
    init_logging();

    // Initialize the async runtime so the #[async_runtime] macro can be used.
    // This function may also be called safely more than once.
    init_async_runtime()?;

    let initialized = init_wallet_environment()?;
    assert!(initialized, "Wallet can only be initialized once");

    Ok(())
}

/// This is called by the public [`init()`] function above.
/// The returned `Result<bool>` is `true` if the wallet was successfully initialized,
/// otherwise it indicates that the wallet was already created.
#[async_runtime]
async fn init_wallet_environment() -> Result<bool> {
    let mut created = false;

    _ = WALLET_API_ENVIRONMENT
        .get_or_try_init(|| async {
            // This closure will only be called if WALLET_API_ENVIRONMENT is currently empty.
            let wallet = init_wallet().await?;
            created = true;

            Ok(WalletApiEnvironment::new(wallet))
        })
        .await?;

    Ok(created)
}

pub fn is_valid_pin(pin: String) -> Vec<u8> {
    let pin_result = PinValidationResult::from(validate_pin(&pin));
    bincode::serialize(&pin_result).unwrap()
}

#[async_runtime]
pub async fn set_lock_stream(sink: StreamSink<bool>) -> Result<()> {
    let sink = ClosingStreamSink::from(sink);

    wallet()
        .write()
        .await
        .set_lock_callback(move |locked| sink.add(locked));

    Ok(())
}

#[async_runtime]
pub async fn clear_lock_stream() -> Result<()> {
    wallet().write().await.clear_lock_callback();

    Ok(())
}

#[async_runtime]
pub async fn set_configuration_stream(sink: StreamSink<FlutterConfiguration>) -> Result<()> {
    let sink = ClosingStreamSink::from(sink);

    wallet()
        .write()
        .await
        .set_config_callback(move |config| sink.add(config.into()));

    Ok(())
}

#[async_runtime]
pub async fn clear_configuration_stream() -> Result<()> {
    wallet().write().await.clear_config_callback();

    Ok(())
}

#[async_runtime]
pub async fn unlock_wallet(pin: String) -> Vec<u8> {
    let wallet_env = wallet_environment();
    let mut wallet = wallet_env.wallet.write().await;

    let unlock_result = wallet.unlock(pin).await;
    let wallet_unlock_result = WalletUnlockResult::from(unlock_result);

    bincode::serialize(&wallet_unlock_result).unwrap()
}

#[async_runtime]
pub async fn lock_wallet() -> Result<()> {
    let wallet_env = wallet_environment();
    let mut wallet = wallet_env.wallet.write().await;

    wallet.lock();

    Ok(())
}

#[async_runtime]
pub async fn has_registration() -> Result<bool> {
    let has_registration = wallet().read().await.has_registration();
    Ok(has_registration)
}

#[async_runtime]
pub async fn register(pin: String) -> Result<()> {
    let wallet_env = wallet_environment();
    let mut wallet = wallet_env.wallet.write().await;

    wallet.register(pin).await?;

    Ok(())
}

#[async_runtime]
pub async fn get_digid_auth_url() -> Result<String> {
    // TODO: Replace with real implementation.
    Ok("https://example.com".to_string())
}

#[async_runtime]
pub async fn process_uri(uri: String, sink: StreamSink<Vec<u8>>) -> Result<()> {
    // TODO: The code below is POC sample code, to be replace with a real implementation.
    let sink = ClosingStreamSink::from(sink);

    if uri.contains("authentication") {
        let auth_event = UriFlowEvent::DigidAuth {
            state: DigidState::Authenticating,
        };
        sink.add(bincode::serialize(&auth_event).unwrap());
        sleep(Duration::from_secs(5));
        if uri.contains("success") {
            let success_event = UriFlowEvent::DigidAuth {
                state: DigidState::Success,
            };
            sink.add(bincode::serialize(&success_event).unwrap());
        } else {
            let error_event = UriFlowEvent::DigidAuth {
                state: DigidState::Error,
            };
            sink.add(bincode::serialize(&error_event).unwrap());
        }
    } else {
        return Err(anyhow!("Sample error, this closes the stream on the flutter side."));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_is_valid_pin(pin: &str) -> bool {
        let serialized_pin_result = is_valid_pin(pin.to_owned());
        let pin_result = bincode::deserialize(&serialized_pin_result).unwrap();
        matches!(pin_result, PinValidationResult::Ok)
    }

    #[test]
    fn check_valid_pin() {
        assert!(test_is_valid_pin("142032"));
    }

    #[test]
    fn check_invalid_pin() {
        assert!(!test_is_valid_pin("sdfioj"));
    }
}
