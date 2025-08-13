use flutter_rust_bridge::frb;
use flutter_rust_bridge::setup_default_user_utils;
use tokio::sync::OnceCell;
use tokio::sync::RwLock;
use url::Url;

use flutter_api_macros::flutter_api_error;
use wallet::DisclosureUriSource;
use wallet::UnlockMethod;
use wallet::Wallet;
use wallet::errors::WalletInitError;
use wallet::utils::version_string;

use crate::frb_generated::StreamSink;
use crate::logging::init_logging;
use crate::models::attestation::AttestationPresentation;
use crate::models::config::FlutterConfiguration;
use crate::models::disclosure::AcceptDisclosureResult;
use crate::models::disclosure::StartDisclosureResult;
use crate::models::instruction::DisclosureBasedIssuanceResult;
use crate::models::instruction::WalletInstructionResult;
use crate::models::pin::PinValidationResult;
use crate::models::uri::IdentifyUriResult;
use crate::models::version_state::FlutterVersionState;
use crate::models::wallet_event::WalletEvent;
use crate::sentry::init_sentry;

static WALLET: OnceCell<RwLock<Wallet>> = OnceCell::const_new();

fn wallet() -> &'static RwLock<Wallet> {
    WALLET
        .get()
        .expect("Wallet must be initialized. Please execute `init()` first.")
}

#[frb(init)]
#[flutter_api_error]
pub async fn init() -> anyhow::Result<()> {
    if !is_initialized() {
        // Initialize platform specific logging and set the log level.
        // As creating the wallet below could fail and init() could be called again,
        // init_logging() should not fail when being called more than once.
        init_logging();

        // Setup logging to console and enable RUST_BACKTRACE to be caught on panics (but not errors) for Sentry.
        setup_default_user_utils();
        unsafe {
            std::env::set_var("RUST_LIB_BACKTRACE", "0");
        }

        // Initialize Sentry for Rust panics.
        // This MUST be called before initializing the async runtime.
        init_sentry();

        create_wallet().await?;
    }

    Ok(())
}

pub fn is_initialized() -> bool {
    WALLET.initialized()
}

/// This is called by the public [`init()`] function above.
/// The returned `Result<bool>` is `true` if the wallet was successfully initialized,
/// otherwise it indicates that the wallet was already created.
async fn create_wallet() -> Result<(), WalletInitError> {
    _ = WALLET
        .get_or_try_init(|| async {
            // This closure will only be called if WALLET_API_ENVIRONMENT is currently empty.
            let wallet = Wallet::init_all().await?;
            Ok::<_, WalletInitError>(RwLock::new(wallet))
        })
        .await?;

    Ok(())
}

#[flutter_api_error]
pub fn is_valid_pin(pin: String) -> anyhow::Result<PinValidationResult> {
    let result = wallet::validate_pin(&pin).into();

    Ok(result)
}

pub async fn set_lock_stream(sink: StreamSink<bool>) {
    wallet().write().await.set_lock_callback(Box::new(move |locked| {
        let _ = sink.add(locked);
    }));
}

pub async fn clear_lock_stream() {
    wallet().write().await.clear_lock_callback();
}

pub async fn set_configuration_stream(sink: StreamSink<FlutterConfiguration>) {
    wallet().read().await.set_config_callback(Box::new(move |config| {
        let _ = sink.add(config.as_ref().into());
    }));
}

pub async fn clear_configuration_stream() {
    wallet().read().await.clear_config_callback();
}

pub async fn set_version_state_stream(sink: StreamSink<FlutterVersionState>) {
    wallet().read().await.set_version_state_callback(Box::new(move |state| {
        let _ = sink.add(state.into());
    }));
}

pub async fn clear_version_state_stream() {
    wallet().read().await.clear_version_state_callback();
}

pub async fn set_attestations_stream(sink: StreamSink<Vec<AttestationPresentation>>) -> anyhow::Result<()> {
    wallet()
        .write()
        .await
        .set_attestations_callback(Box::new(move |attestations| {
            let _ = sink.add(attestations.into_iter().map(AttestationPresentation::from).collect());
        }))
        .await?;

    Ok(())
}

pub async fn clear_attestations_stream() {
    wallet().write().await.clear_attestations_callback();
}

#[flutter_api_error]
pub async fn set_recent_history_stream(sink: StreamSink<Vec<WalletEvent>>) -> anyhow::Result<()> {
    wallet()
        .write()
        .await
        .set_recent_history_callback(Box::new(move |events| {
            let recent_history = events.into_iter().map(WalletEvent::from).collect();

            let _ = sink.add(recent_history);
        }))
        .await?;

    Ok(())
}

pub async fn clear_recent_history_stream() {
    wallet().write().await.clear_recent_history_callback();
}

#[flutter_api_error]
pub async fn unlock_wallet(pin: String) -> anyhow::Result<WalletInstructionResult> {
    let mut wallet = wallet().write().await;

    let result = wallet.unlock(pin).await.try_into()?;

    Ok(result)
}

pub async fn lock_wallet() {
    let mut wallet = wallet().write().await;

    wallet.lock();
}

#[flutter_api_error]
pub async fn check_pin(pin: String) -> anyhow::Result<WalletInstructionResult> {
    let wallet = wallet().read().await;

    let result = wallet.check_pin(pin).await.try_into()?;

    Ok(result)
}

#[flutter_api_error]
pub async fn change_pin(old_pin: String, new_pin: String) -> anyhow::Result<WalletInstructionResult> {
    let mut wallet = wallet().write().await;

    let result = wallet.begin_change_pin(old_pin, new_pin).await.try_into()?;

    Ok(result)
}

#[flutter_api_error]
pub async fn continue_change_pin(pin: &str) -> anyhow::Result<WalletInstructionResult> {
    let wallet = wallet().read().await;

    let result = wallet.continue_change_pin(pin).await.try_into()?;

    Ok(result)
}

pub async fn has_registration() -> bool {
    wallet().read().await.has_registration()
}

#[flutter_api_error]
pub async fn register(pin: &str) -> anyhow::Result<()> {
    let mut wallet = wallet().write().await;

    wallet.register(pin).await?;

    Ok(())
}

#[flutter_api_error]
pub async fn identify_uri(uri: String) -> anyhow::Result<IdentifyUriResult> {
    let wallet = wallet().read().await;

    let identify_uri_result = wallet.identify_uri(&uri).try_into()?;

    Ok(identify_uri_result)
}

#[flutter_api_error]
pub async fn create_pid_issuance_redirect_uri() -> anyhow::Result<String> {
    let mut wallet = wallet().write().await;

    let auth_url = wallet.create_pid_issuance_auth_url().await?;

    Ok(auth_url.into())
}

#[flutter_api_error]
pub async fn create_pid_renewal_redirect_uri() -> anyhow::Result<String> {
    // TODO: Implement as part of PVW-4586
    Ok("pid_renewal_url".into())
}

#[flutter_api_error]
pub async fn create_pin_recovery_redirect_uri() -> anyhow::Result<String> {
    // TODO: Implement as part of PVW-4587
    Ok("pin_recovery_url".into())
}

#[flutter_api_error]
pub async fn continue_pin_recovery(uri: String) -> anyhow::Result<()> {
    // TODO: Implement as part of PVW-4587
    println!("Recovery URI: {uri}");
    Ok(())
}

#[flutter_api_error]
pub async fn complete_pin_recovery(pin: String) -> anyhow::Result<WalletInstructionResult> {
    // TODO: Implement as part of PVW-4587
    println!("PIN length: {}", pin.len());
    Ok(WalletInstructionResult::Ok)
}

#[flutter_api_error]
pub async fn cancel_pin_recovery() -> anyhow::Result<()> {
    // TODO: Implement as part of PVW-4587
    Ok(())
}

#[flutter_api_error]
pub async fn cancel_issuance() -> anyhow::Result<()> {
    let mut wallet = wallet().write().await;

    wallet.cancel_issuance().await?;

    Ok(())
}

#[flutter_api_error]
pub async fn continue_pid_issuance(uri: String) -> anyhow::Result<Vec<AttestationPresentation>> {
    let url = Url::parse(&uri)?;

    let mut wallet = wallet().write().await;

    let wallet_attestations = wallet.continue_pid_issuance(url).await?;
    let attestations = wallet_attestations
        .into_iter()
        .map(AttestationPresentation::from)
        .collect();

    Ok(attestations)
}

#[flutter_api_error]
pub async fn accept_issuance(pin: String) -> anyhow::Result<WalletInstructionResult> {
    let mut wallet = wallet().write().await;

    let result = wallet.accept_issuance(pin).await.try_into()?;

    Ok(result)
}

#[flutter_api_error]
pub async fn has_active_issuance_session() -> anyhow::Result<bool> {
    let wallet = wallet().read().await;

    let has_active_session = wallet.has_active_issuance_session()?;

    Ok(has_active_session)
}

#[flutter_api_error]
pub async fn start_disclosure(uri: String, is_qr_code: bool) -> anyhow::Result<StartDisclosureResult> {
    let url = Url::parse(&uri)?;

    let mut wallet = wallet().write().await;

    let result = wallet
        .start_disclosure(&url, DisclosureUriSource::new(is_qr_code))
        .await
        .try_into()?;

    Ok(result)
}

#[flutter_api_error]
pub async fn cancel_disclosure() -> anyhow::Result<Option<String>> {
    let mut wallet = wallet().write().await;

    let return_url = wallet.cancel_disclosure().await?.map(String::from);

    Ok(return_url)
}

#[flutter_api_error]
pub async fn accept_disclosure(pin: String) -> anyhow::Result<AcceptDisclosureResult> {
    let mut wallet = wallet().write().await;

    let result = wallet.accept_disclosure(pin).await.try_into()?;

    Ok(result)
}

#[flutter_api_error]
pub async fn has_active_disclosure_session() -> anyhow::Result<bool> {
    let wallet = wallet().read().await;

    let has_active_session = wallet.has_active_disclosure_session()?;

    Ok(has_active_session)
}

#[flutter_api_error]
pub async fn continue_disclosure_based_issuance(pin: String) -> anyhow::Result<DisclosureBasedIssuanceResult> {
    let mut wallet = wallet().write().await;

    let result = wallet.continue_disclosure_based_issuance(pin).await.try_into()?;

    Ok(result)
}

#[flutter_api_error]
pub async fn is_biometric_unlock_enabled() -> anyhow::Result<bool> {
    let wallet = wallet().read().await;

    let is_biometrics_enabled = wallet.unlock_method().await?.has_biometrics();

    Ok(is_biometrics_enabled)
}

#[flutter_api_error]
pub async fn set_biometric_unlock(enable: bool) -> anyhow::Result<()> {
    let mut wallet = wallet().write().await;

    let unlock_method = if enable {
        UnlockMethod::PinCodeAndBiometrics
    } else {
        UnlockMethod::PinCode
    };
    wallet.set_unlock_method(unlock_method).await?;

    Ok(())
}

#[flutter_api_error]
pub async fn unlock_wallet_with_biometrics() -> anyhow::Result<()> {
    let mut wallet = wallet().write().await;

    wallet.unlock_without_pin().await?;

    Ok(())
}

#[flutter_api_error]
pub async fn get_history() -> anyhow::Result<Vec<WalletEvent>> {
    let wallet = wallet().read().await;
    let history = wallet.get_history().await?;
    let history = history.into_iter().map(WalletEvent::from).collect();
    Ok(history)
}

#[flutter_api_error]
pub async fn get_history_for_card(attestation_id: String) -> anyhow::Result<Vec<WalletEvent>> {
    let wallet = wallet().read().await;
    let history = wallet.get_history_for_card(&attestation_id).await?;
    let history = history.into_iter().map(WalletEvent::from).collect();
    Ok(history)
}

#[flutter_api_error]
pub async fn reset_wallet() -> anyhow::Result<()> {
    wallet().write().await.reset().await?;

    Ok(())
}

pub fn get_version_string() -> String {
    version_string()
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
