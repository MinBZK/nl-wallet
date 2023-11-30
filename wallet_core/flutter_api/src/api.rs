use anyhow::Result;
use flutter_rust_bridge::StreamSink;
use tokio::sync::{OnceCell, RwLock};
use url::Url;

use flutter_api_macros::{async_runtime, flutter_api_error};
use wallet::{self, errors::WalletInitError, x509::CertificateError, Wallet};

use crate::{
    async_runtime::init_async_runtime,
    logging::init_logging,
    models::{
        card::{Card, CardAttribute, CardValue, LocalizedString},
        config::FlutterConfiguration,
        disclosure::{Organization, RequestPolicy, RequestedCard, StartDisclosureResult},
        instruction::WalletInstructionResult,
        pin::PinValidationResult,
        uri::IdentifyUriResult,
        wallet_event::{DisclosureStatus, WalletEvent},
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
        .set_config_callback(move |config| sink.add((&config.lock_timeouts).into()));
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
pub async fn identify_uri(uri: String) -> Result<IdentifyUriResult> {
    let wallet = wallet().read().await;

    let identify_uri_result = wallet.identify_uri(&uri).try_into()?;

    Ok(identify_uri_result)
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

#[async_runtime]
#[flutter_api_error]
pub async fn continue_pid_issuance(uri: String) -> Result<Vec<Card>> {
    let url = Url::parse(&uri)?;

    let mut wallet = wallet().write().await;

    let documents = wallet.continue_pid_issuance(&url).await?;

    let cards = documents.into_iter().map(Card::from).collect();

    Ok(cards)
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
#[flutter_api_error]
pub async fn start_disclosure(uri: String) -> Result<StartDisclosureResult> {
    let url = Url::parse(&uri)?;

    let mut wallet = wallet().write().await;

    let result = wallet.start_disclosure(&url).await.try_into()?;

    Ok(result)
}

#[async_runtime]
#[flutter_api_error]
pub async fn cancel_disclosure() -> Result<()> {
    let mut wallet = wallet().write().await;

    wallet.cancel_disclosure().await?;

    Ok(())
}

#[async_runtime]
#[flutter_api_error]
pub async fn accept_disclosure(pin: String) -> Result<WalletInstructionResult> {
    // TODO: implement.

    if pin == "000000" {
        return Ok(WalletInstructionResult::IncorrectPin {
            leftover_attempts: 3,
            is_final_attempt: false,
        });
    } else if pin == "111111" {
        return Ok(WalletInstructionResult::Timeout { timeout_millis: 10_000 });
    } else if pin == "222222" {
        return Ok(WalletInstructionResult::Blocked {});
    }

    Ok(WalletInstructionResult::Ok)
}

// TODO remove when events for disclosure are sent
fn get_hardcoded_disclosure_events() -> Vec<WalletEvent> {
    vec![WalletEvent::Disclosure {
        relying_party: Organization {
            legal_name: vec![LocalizedString {
                language: "nl".to_owned(),
                value: "RP Legal Name".to_owned(),
            }],
            display_name: vec![LocalizedString {
                language: "nl".to_owned(),
                value: "RP Display Name".to_owned(),
            }],
            description: vec![LocalizedString {
                language: "nl".to_owned(),
                value: "RP Description".to_owned(),
            }],
            category: vec![LocalizedString {
                language: "nl".to_owned(),
                value: "Overheid".to_owned(),
            }],
            image: None,
            web_url: Some("https://example.org".to_owned()),
            kvk: Some("1234 5678".to_owned()),
            city: Some(vec![LocalizedString {
                language: "nl".to_owned(),
                value: "RP City".to_owned(),
            }]),
            country_code: Some("nl".to_owned()),
            department: None,
        },
        purpose: vec![LocalizedString {
            language: "nl".to_owned(),
            value: "RP Purpose".to_owned(),
        }],
        requested_cards: vec![RequestedCard {
            doc_type: "com.example.pid".to_string(),
            attributes: vec![CardAttribute {
                key: "sample".to_string(),
                labels: vec![LocalizedString {
                    language: "en".to_string(),
                    value: "Sample label".to_string(),
                }],
                value: CardValue::String {
                    value: "Sample value".to_string(),
                },
            }],
        }],
        date_time: "2023-11-06T15:19:30+0100".to_string(),
        status: DisclosureStatus::Success,
        request_policy: RequestPolicy {
            data_storage_duration_in_minutes: Some(60 * 24 * 365),
            data_shared_with_third_parties: false,
            data_deletion_possible: false,
            policy_url: "".to_string(),
        },
    }]
}

#[async_runtime]
#[flutter_api_error]
pub async fn get_history() -> Result<Vec<WalletEvent>> {
    let wallet = wallet().read().await;
    let history = wallet.get_history().await?;
    let mut history = history
        .into_iter()
        .map(WalletEvent::try_from)
        .collect::<Result<Vec<_>, CertificateError>>()?;

    // at the moment there are no disclosure events yet, so add one here
    // TODO remove when disclosure events are implemented
    let mut hardcoded_history = get_hardcoded_disclosure_events();
    history.append(&mut hardcoded_history);
    Ok(history)
}

#[async_runtime]
#[flutter_api_error]
pub async fn get_history_for_card(doc_type: String) -> Result<Vec<WalletEvent>> {
    let wallet = wallet().read().await;
    let history = wallet.get_history_for_card(&doc_type).await?;
    let mut history = history
        .into_iter()
        .map(WalletEvent::try_from)
        .collect::<Result<Vec<_>, CertificateError>>()?;

    // at the moment there are no disclosure events yet, so add one here
    // TODO remove when disclosure events are implemented
    let mut hardcoded_history = get_hardcoded_disclosure_events();
    history.append(&mut hardcoded_history);
    Ok(history)
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
