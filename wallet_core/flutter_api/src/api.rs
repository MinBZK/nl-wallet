use std::sync::Mutex;

use anyhow::Result;
use flutter_rust_bridge::StreamSink;
use once_cell::sync::Lazy;
use tokio::sync::{OnceCell, RwLock};
use tracing::{info, warn};
use url::Url;

use flutter_api_macros::{async_runtime, flutter_api_error};
use wallet::{
    init_wallet, validate_pin,
    wallet::{RedirectUriType, WalletInitError},
    Wallet,
};

use crate::{
    async_runtime::init_async_runtime,
    logging::init_logging,
    models::{
        card::{mock_cards, Card},
        config::FlutterConfiguration,
        pin::PinValidationResult,
        process_uri_event::{PidIssuanceEvent, ProcessUriEvent},
        unlock::WalletUnlockResult,
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
pub async fn set_lock_stream(sink: StreamSink<bool>) {
    let sink = ClosingStreamSink::from(sink);

    wallet().write().await.set_lock_callback(move |locked| sink.add(locked));
}

#[async_runtime]
pub async fn clear_lock_stream() {
    wallet().write().await.clear_lock_callback();
}

#[async_runtime]
pub async fn set_configuration_stream(sink: StreamSink<FlutterConfiguration>) {
    let sink = ClosingStreamSink::from(sink);

    wallet()
        .write()
        .await
        .set_config_callback(move |config| sink.add(config.into()));
}

#[async_runtime]
pub async fn clear_configuration_stream() {
    wallet().write().await.clear_config_callback();
}

type CardsCallback = Box<dyn FnMut(Vec<Card>) + Send>;
static CARDS_CALLBACK: Lazy<Mutex<Option<CardsCallback>>> = Lazy::new(|| Mutex::new(None));

#[async_runtime]
pub async fn set_cards_stream(sink: StreamSink<Vec<Card>>) {
    let sink = ClosingStreamSink::from(sink);

    CARDS_CALLBACK.lock().unwrap().replace(Box::new(move |cards| {
        sink.add(cards);
    }));
}

#[async_runtime]
pub async fn clear_cards_stream() {
    CARDS_CALLBACK.lock().unwrap().take();
}

#[async_runtime]
#[flutter_api_error]
pub async fn unlock_wallet(pin: String) -> Result<WalletUnlockResult> {
    let mut wallet = wallet().write().await;

    let result = wallet.unlock(pin).await.try_into()?;

    Ok(result)
}

#[async_runtime]
pub async fn lock_wallet() {
    let mut wallet = wallet().write().await;

    wallet.lock();
}

#[async_runtime]
pub async fn has_registration() -> bool {
    wallet().read().await.has_registration()
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
pub async fn create_pid_issuance_redirect_uri() -> Result<String> {
    let mut wallet = wallet().write().await;

    let auth_url = wallet.create_pid_issuance_redirect_uri().await?;

    Ok(auth_url.into())
}

#[async_runtime]
pub async fn cancel_pid_issuance() {
    let mut wallet = wallet().write().await;

    wallet.cancel_pid_issuance()
}

#[async_runtime]
#[flutter_api_error]
pub async fn reject_pid_issuance() -> Result<()> {
    let mut wallet = wallet().write().await;

    wallet.reject_pid_issuance().await?;

    Ok(())
}

// todo: handle inner instructions errors (regarding PIN) similarly to unlock_wallet above.
#[async_runtime]
#[flutter_api_error]
pub async fn accept_pid_issuance(pin: String) -> Result<()> {
    let mut wallet = wallet().write().await;

    wallet.accept_pid_issuance(pin).await?;

    // If PID issuance succeeded, send the mock cards through the cards stream.
    if let Some(cards_callback) = CARDS_CALLBACK.lock().unwrap().as_mut() {
        cards_callback(mock_cards());
    }

    Ok(())
}

// Note that any return value from this function (success or error) is ignored in Flutter!
#[async_runtime]
pub async fn process_uri(uri: String, sink: StreamSink<ProcessUriEvent>) {
    let sink = ClosingStreamSink::from(sink);

    // Parse the URI we have received from Flutter.
    let url = match Url::parse(&uri) {
        Ok(url) => url,
        Err(_) => {
            // If URL parsing fails, this is probably an error on the Flutter side.
            // Rather than panicking we just return that we do not know this URI.
            sink.add(ProcessUriEvent::UnknownUri);

            return;
        }
    };

    // Have the wallet identify the type of redirect URI.
    // Note that the obtained read lock only exists temporarily.
    let uri_type = wallet().read().await.identify_redirect_uri(&url);

    let final_event = match uri_type {
        // This is a PID issuance redirect URI.
        RedirectUriType::PidIssuance => {
            // Send an event on the stream to indicate that we are in the PID issuance flow.
            let auth_event = ProcessUriEvent::PidIssuance {
                event: PidIssuanceEvent::Authenticating,
            };
            sink.add(auth_event);

            // Have the wallet actually process the redirect URI.
            let event = process_pid_issuance_redirect_uri(&url).await;

            ProcessUriEvent::PidIssuance { event }
        }
        // The wallet does not recognise the redirect URI.
        RedirectUriType::Unknown => ProcessUriEvent::UnknownUri,
    };

    sink.add(final_event);
}

async fn process_pid_issuance_redirect_uri(url: &Url) -> PidIssuanceEvent {
    let mut wallet = wallet().write().await;

    wallet.continue_pid_issuance(url).await.map_or_else(
        |error| {
            // Log the error, since this is not caught by the `#[flutter_api_error]` macro.
            warn!("PID issuance error: {}", error);
            info!("PID issuance error details: {:?}", error);

            // Then convert then error to JSON, wrapped inside a `PidIssuanceEvent::Error`.
            error.into()
        },
        |_mdocs| PidIssuanceEvent::Success {
            preview_cards: mock_cards(), // TODO: actually convert mdocs to card
        },
    )
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
