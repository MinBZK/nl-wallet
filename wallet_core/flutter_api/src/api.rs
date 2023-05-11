use anyhow::{Ok, Result};
use tokio::sync::{OnceCell, RwLock};

use macros::async_runtime;
use wallet::{init_wallet, pin::validation::validate_pin, Wallet};

use crate::{async_runtime::init_async_runtime, models::pin::PinValidationResult};

static WALLET: OnceCell<RwLock<Wallet>> = OnceCell::const_new();

fn wallet() -> &'static RwLock<Wallet> {
    WALLET
        .get()
        .expect("Wallet must be initialized. Please execute `init()` first.")
}

pub fn init() -> Result<bool> {
    // Initialize the async runtime so the #[async_runtime] macro can be used.
    // As creating the wallet below could fail and init() could be called again,
    // init_async_runtime() should not fail when being called more than once.
    init_async_runtime()?;

    // Panic if create_wallet() returns None.
    let has_registration = create_wallet()?.expect("Wallet may only be initialized once.");

    Ok(has_registration)
}

/// This is called by the public [`init()`] function above.
/// The returned `Option<bool>` is `None` if the wallet was already initialized,
/// otherwise it indicates if the wallet contains a previously saved registration.
#[async_runtime]
async fn create_wallet() -> Result<Option<bool>> {
    let mut created_has_registration: Option<bool> = None;

    _ = WALLET
        .get_or_try_init(|| async {
            // This closure will only be called if WALLET is currently empty.
            let wallet = init_wallet().await?;
            created_has_registration.replace(wallet.has_registration());

            Ok(RwLock::new(wallet))
        })
        .await?;

    // This will be None if the async block above did not execute.
    Ok(created_has_registration)
}

pub fn is_valid_pin(pin: String) -> Vec<u8> {
    let pin_result = PinValidationResult::from(validate_pin(&pin));
    bincode::serialize(&pin_result).unwrap()
}

#[async_runtime]
pub async fn register(pin: String) -> Result<()> {
    // TODO return differentiated errors?
    wallet().write().await.register(pin).await
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
