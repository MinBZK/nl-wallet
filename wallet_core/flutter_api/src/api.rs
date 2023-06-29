use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Result};
use flutter_rust_bridge::StreamSink;
use tokio::sync::{OnceCell, RwLock};

use flutter_api_macros::{async_runtime, flutter_api_error};
use wallet::{init_wallet, validate_pin, wallet::WalletInitError, Wallet};

use crate::{
    async_runtime::init_async_runtime,
    logging::init_logging,
    models::{
        config::FlutterConfiguration,
        pin::PinValidationResult,
        unlock::WalletUnlockResult,
        uri_flow_event::{DigidState, UriFlowEvent},
    },
    stream::ClosingStreamSink,
};

static WALLET: OnceCell<RwLock<Wallet>> = OnceCell::const_new();

fn wallet() -> &'static RwLock<Wallet> {
    WALLET
        .get()
        .expect("Wallet must be initialized. Please execute `init()` first.")
}

#[flutter_api_error]
pub fn init() -> Result<()> {
    // Initialize platform specific logging and set the log level.
    // As creating the wallet below could fail and init() could be called again,
    // init_logging() should not fail when being called more than once.
    init_logging();

    // Initialize the async runtime so the #[async_runtime] macro can be used.
    // This function may also be called safely more than once.
    init_async_runtime();

    let initialized = create_wallet()?;
    assert!(initialized, "Wallet can only be initialized once");

    Ok(())
}

pub fn is_initialized() -> bool {
    WALLET.initialized()
}

/// This is called by the public [`init()`] function above.
/// The returned `Result<bool>` is `true` if the wallet was successfully initialized,
/// otherwise it indicates that the wallet was already created.
#[async_runtime]
async fn create_wallet() -> std::result::Result<bool, WalletInitError> {
    let mut created = false;

    _ = WALLET
        .get_or_try_init(|| async {
            // This closure will only be called if WALLET_API_ENVIRONMENT is currently empty.
            let wallet = init_wallet().await?;
            created = true;

            Ok::<_, WalletInitError>(RwLock::new(wallet))
        })
        .await?;

    Ok(created)
}

#[flutter_api_error]
pub fn is_valid_pin(pin: String) -> Result<PinValidationResult> {
    let result = validate_pin(&pin).into();

    Ok(result)
}

#[async_runtime]
#[flutter_api_error]
pub async fn set_lock_stream(sink: StreamSink<bool>) -> Result<()> {
    let sink = ClosingStreamSink::from(sink);

    wallet().write().await.set_lock_callback(move |locked| sink.add(locked));

    Ok(())
}

#[async_runtime]
#[flutter_api_error]
pub async fn clear_lock_stream() -> Result<()> {
    wallet().write().await.clear_lock_callback();

    Ok(())
}

#[async_runtime]
#[flutter_api_error]
pub async fn set_configuration_stream(sink: StreamSink<FlutterConfiguration>) -> Result<()> {
    let sink = ClosingStreamSink::from(sink);

    wallet()
        .write()
        .await
        .set_config_callback(move |config| sink.add(config.into()));

    Ok(())
}

#[async_runtime]
#[flutter_api_error]
pub async fn clear_configuration_stream() -> Result<()> {
    wallet().write().await.clear_config_callback();

    Ok(())
}

#[async_runtime]
#[flutter_api_error]
pub async fn unlock_wallet(pin: String) -> Result<WalletUnlockResult> {
    let mut wallet = wallet().write().await;

    let result = wallet.unlock(pin).await.try_into()?;

    Ok(result)
}

#[async_runtime]
#[flutter_api_error]
pub async fn lock_wallet() -> Result<()> {
    let mut wallet = wallet().write().await;

    wallet.lock();

    Ok(())
}

#[async_runtime]
#[flutter_api_error]
pub async fn has_registration() -> Result<bool> {
    let has_registration = wallet().read().await.has_registration();
    Ok(has_registration)
}

#[async_runtime]
#[flutter_api_error]
pub async fn register(pin: String) -> Result<()> {
    let mut wallet = wallet().write().await;

    wallet.register(pin).await?;

    Ok(())
}

#[async_runtime]
#[flutter_api_error]
pub async fn get_digid_auth_url() -> Result<String> {
    // TODO: Replace with real implementation.
    Ok("https://example.com".to_string())
}

#[async_runtime]
#[flutter_api_error]
pub async fn process_uri(uri: String, sink: StreamSink<UriFlowEvent>) -> Result<()> {
    // TODO: The code below is POC sample code, to be replace with a real implementation.
    let sink = ClosingStreamSink::from(sink);

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
