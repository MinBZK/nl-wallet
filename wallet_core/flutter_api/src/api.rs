use anyhow::Result;
use flutter_rust_bridge::StreamSink;
use tokio::sync::{OnceCell, RwLock};
use tracing::{info, warn};
use url::Url;

use flutter_api_macros::{async_runtime, flutter_api_error};
use wallet::{
    self,
    errors::{UriIdentificationError, WalletInitError, WalletUnlockError},
    UriType, Wallet,
};

use crate::{
    async_runtime::init_async_runtime,
    logging::init_logging,
    models::{
        card::{Card, CardAttribute, CardValue, LocalizedString},
        config::FlutterConfiguration,
        disclosure::{MissingAttribute, RelyingParty, RequestedCard},
        instruction::WalletInstructionResult,
        pin::PinValidationResult,
        process_uri_event::{DisclosureEvent, PidIssuanceEvent, ProcessUriEvent},
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
            let wallet = Wallet::init_all().await?;
            created = true;

            Ok::<_, WalletInitError>(RwLock::new(wallet))
        })
        .await?;

    Ok(created)
}

#[flutter_api_error]
pub fn is_valid_pin(pin: String) -> Result<PinValidationResult> {
    let result = wallet::validate_pin(&pin).into();

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

#[async_runtime]
pub async fn set_cards_stream(sink: StreamSink<Vec<Card>>) -> Result<()> {
    let sink = ClosingStreamSink::from(sink);

    wallet()
        .write()
        .await
        .set_documents_callback(move |documents| {
            let cards = documents.into_iter().map(|document| document.into()).collect();

            sink.add(cards);
        })
        .await?;

    Ok(())
}

#[async_runtime]
pub async fn clear_cards_stream() {
    wallet().write().await.clear_documents_callback();
}

#[async_runtime]
#[flutter_api_error]
pub async fn unlock_wallet(pin: String) -> Result<WalletInstructionResult> {
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

    let auth_url = wallet.create_pid_issuance_auth_url().await?;

    Ok(auth_url.into())
}

#[async_runtime]
#[flutter_api_error]
pub async fn cancel_pid_issuance() -> Result<()> {
    let mut wallet = wallet().write().await;

    wallet.cancel_pid_issuance()?;

    Ok(())
}

// Note that any return value from this function (success or error) is ignored in Flutter!
#[async_runtime]
pub async fn process_uri(uri: String, sink: StreamSink<ProcessUriEvent>) {
    let sink = ClosingStreamSink::from(sink);

    // Parse the URI we have received from Flutter and identify the type of
    // redirect URI. Note that the obtained read lock only exists temporarily.
    let uri_type = match wallet().read().await.identify_uri(&uri) {
        Ok(uri_type) => uri_type,
        Err(error) => {
            // If URL parsing fails, this is probably an error on the Flutter side.
            // Rather than panicking we just return that we do not know this URI and log a warning.
            if let UriIdentificationError::Parse(error) = error {
                warn!("Redirect URI error: {}", error);
            }

            sink.add(ProcessUriEvent::UnknownUri);

            return;
        }
    };

    let final_event = match uri_type {
        // This is a PID issuance redirect URI.
        UriType::PidIssuance(url) => {
            // Send an event on the stream to indicate that we are in the PID issuance flow.
            let auth_event = ProcessUriEvent::PidIssuance {
                event: PidIssuanceEvent::Authenticating,
            };
            sink.add(auth_event);

            // Have the wallet actually process the redirect URI.
            let event = process_pid_issuance_redirect_uri(&url).await;

            ProcessUriEvent::PidIssuance { event }
        }
        // Start a disclosure flow.
        UriType::Disclosure(url) => {
            let fetching_event = ProcessUriEvent::Disclosure {
                event: DisclosureEvent::FetchingRequest,
            };
            sink.add(fetching_event);

            // Have the wallet process the disclosure URI.
            let event = process_disclosure_uri(&url);

            ProcessUriEvent::Disclosure { event }
        }
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
        |documents| PidIssuanceEvent::Success {
            preview_cards: documents.into_iter().map(Card::from).collect(),
        },
    )
}

// TODO: actually talk to Wallet for fetching attribute request.
fn process_disclosure_uri(url: &Url) -> DisclosureEvent {
    match url.as_str() {
        "walletdebuginteraction://wallet.edi.rijksoverheid.nl/disclosure/request" => DisclosureEvent::Request {
            relying_party: RelyingParty {
                name: "The Relying Party".to_string(),
            },
            requested_cards: vec![RequestedCard {
                doc_type: "com.example.pid".to_string(),
                attributes: vec![CardAttribute {
                    key: "given_name".to_string(),
                    labels: vec![
                        LocalizedString {
                            language: "en".to_string(),
                            value: "First name".to_string(),
                        },
                        LocalizedString {
                            language: "nl".to_string(),
                            value: "Voornaam".to_string(),
                        },
                    ],
                    value: CardValue::String {
                        value: "Willeke Liselotte".to_string(),
                    },
                }],
            }],
        },
        "walletdebuginteraction://wallet.edi.rijksoverheid.nl/disclosure/missing" => {
            DisclosureEvent::RequestAttributesMissing {
                relying_party: RelyingParty {
                    name: "Other Relying Party".to_string(),
                },
                missing_attributes: vec![MissingAttribute {
                    labels: vec![
                        LocalizedString {
                            language: "en".to_string(),
                            value: "Email address".to_string(),
                        },
                        LocalizedString {
                            language: "nl".to_string(),
                            value: "E-mailadres".to_string(),
                        },
                    ],
                }],
            }
        }
        _ => WalletUnlockError::NotRegistered.into(),
    }
}

#[async_runtime]
#[flutter_api_error]
async fn cancel_disclosure() -> Result<()> {
    // TODO: implement.

    Ok(())
}

#[async_runtime]
#[flutter_api_error]
async fn accept_disclosure(_pin: String) -> Result<()> {
    // TODO: implement.

    Ok(())
}

#[async_runtime]
#[flutter_api_error]
pub async fn accept_pid_issuance(pin: String) -> Result<WalletInstructionResult> {
    let mut wallet = wallet().write().await;

    let result = wallet.accept_pid_issuance(pin).await.try_into()?;

    Ok(result)
}

#[async_runtime]
#[flutter_api_error]
pub async fn reject_pid_issuance() -> Result<()> {
    let mut wallet = wallet().write().await;

    wallet.reject_pid_issuance().await?;

    Ok(())
}

#[async_runtime]
pub async fn reset_wallet() {
    panic!("Unimplemented: UC 9.4")
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
