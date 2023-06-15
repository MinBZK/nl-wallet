use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Result};
use flutter_rust_bridge::StreamSink;
use tokio::sync::{OnceCell, RwLock};

use flutter_api_macros::async_runtime;
use wallet::{init_wallet, validate_pin, wallet::WalletInitError, Wallet};

use crate::{
    async_runtime::init_async_runtime,
    errors::FlutterApiError,
    logging::init_logging,
    models::{
        pin::PinValidationResult,
        unlock::UnlockResult,
        uri_flow_event::{DigidState, UriFlowEvent},
    },
};

struct WalletApiEnvironment {
    wallet: RwLock<Wallet>,
    lock_sink: StreamSink<bool>,
}

impl WalletApiEnvironment {
    fn new(wallet: Wallet, lock_sink: StreamSink<bool>) -> Self {
        WalletApiEnvironment {
            wallet: RwLock::new(wallet),
            lock_sink,
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

pub fn init(wallet_lock_sink: StreamSink<bool>) -> Result<()> {
    // Initialize platform specific logging and set the log level.
    // As creating the wallet below could fail and init() could be called again,
    // init_logging() should not fail when being called more than once.
    init_logging();

    // Initialize the async runtime so the #[async_runtime] macro can be used.
    // This function may also be called safely more than once.
    init_async_runtime();

    let initialized = init_wallet_environment(wallet_lock_sink)?;
    assert!(initialized, "Wallet can only be initialized once");

    Ok(())
}

/// This is called by the public [`init()`] function above.
/// The returned `Result<bool>` is `true` if the wallet was successfully initialized,
/// otherwise it indicates that the wallet was already created.
#[async_runtime]
async fn init_wallet_environment(lock_sink: StreamSink<bool>) -> Result<bool> {
    let mut created = false;

    _ = WALLET_API_ENVIRONMENT
        .get_or_try_init(|| async {
            // This closure will only be called if WALLET_API_ENVIRONMENT is currently empty.
            let wallet = init_wallet().await?;
            created = true;

            Ok::<_, WalletInitError>(WalletApiEnvironment::new(wallet, lock_sink))
        })
        .await
        .map_err(FlutterApiError::from)?;

    Ok(created)
}

pub fn is_valid_pin(pin: String) -> Result<PinValidationResult> {
    let result = validate_pin(&pin).into();

    Ok(result)
}

#[async_runtime]
pub async fn unlock_wallet(pin: String) -> Result<UnlockResult> {
    let wallet_env = wallet_environment();
    let mut wallet = wallet_env.wallet.write().await;

    let result = wallet.unlock(pin).await.try_into()?;

    if let UnlockResult::Ok = result {
        wallet_env.lock_sink.add(wallet.is_locked());
    }

    Ok(result)
}

#[async_runtime]
pub async fn lock_wallet() -> Result<()> {
    let wallet_env = wallet_environment();
    let mut wallet = wallet_env.wallet.write().await;

    wallet.lock();
    wallet_env.lock_sink.add(wallet.is_locked());

    Ok(())
}

#[async_runtime]
pub async fn has_registration() -> Result<bool> {
    let has_registration = wallet().read().await.has_registration();
    Ok(has_registration)
}

#[async_runtime]
pub async fn register(pin: String) -> Result<()> {
    wallet()
        .write()
        .await
        .register(pin)
        .await
        .map_err(FlutterApiError::from)?;

    Ok(())
}

#[async_runtime]
pub async fn get_digid_auth_url() -> Result<String> {
    // TODO: Replace with real implementation.
    Ok("https://example.com".to_string())
}

#[async_runtime]
pub async fn process_uri(uri: String, sink: StreamSink<UriFlowEvent>) -> Result<()> {
    // TODO: The code below is POC sample code, to be replace with a real implementation.
    if uri.contains("authentication") {
        let auth_event = UriFlowEvent::DigidAuth {
            state: DigidState::Authenticating,
        };
        sink.add(auth_event);
        sleep(Duration::from_secs(5));
        if uri.contains("success") {
            let success_event = UriFlowEvent::DigidAuth {
                state: DigidState::Success,
            };
            sink.add(success_event);
        } else {
            let error_event = UriFlowEvent::DigidAuth {
                state: DigidState::Error,
            };
            sink.add(error_event);
        }
    } else {
        return Err(anyhow!("Sample error, this closes the stream on the flutter side."));
    }
    // TODO: Create newtype and implement Drop trait to automate sink closure.
    sink.close();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_is_valid_pin(pin: &str) -> bool {
        matches!(
            is_valid_pin(pin.to_string()).expect("Could not validate PIN"),
            PinValidationResult::Ok
        )
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
