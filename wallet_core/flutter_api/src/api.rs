use anyhow::{Ok, Result};
use tokio::sync::{OnceCell, RwLock};

use macros::async_runtime;
use wallet::{init_wallet, validate_pin, Wallet};

use crate::{async_runtime::init_async_runtime, logging::init_logging, models::pin::PinValidationResult};

static WALLET: OnceCell<RwLock<Wallet>> = OnceCell::const_new();

fn wallet() -> &'static RwLock<Wallet> {
    WALLET
        .get()
        .expect("Wallet must be initialized. Please execute `init()` first.")
}

pub fn init() -> Result<()> {
    // Initialize platform specific logging and set the log level.
    // As creating the wallet below could fail and init() could be called again,
    // init_logging() should not fail when being called more than once.
    init_logging();

    // Initialize the async runtime so the #[async_runtime] macro can be used.
    // This function may also be called safely more than once.
    init_async_runtime()?;

    let created = create_wallet()?;
    assert!(created, "Wallet can only be initialized once");

    Ok(())
}

/// This is called by the public [`init()`] function above.
/// The returned `Result<bool>` is `true` if the wallet was successfully initialized,
/// otherwise it indicates that the wallet was already created.
#[async_runtime]
async fn create_wallet() -> Result<bool> {
    let mut created = false;

    _ = WALLET
        .get_or_try_init(|| async {
            // This closure will only be called if WALLET is currently empty.
            let wallet = init_wallet().await?;
            created = true;

            Ok(RwLock::new(wallet))
        })
        .await?;

    Ok(created)
}

pub fn is_valid_pin(pin: String) -> Vec<u8> {
    let pin_result = PinValidationResult::from(validate_pin(&pin));
    bincode::serialize(&pin_result).unwrap()
}

#[async_runtime]
pub async fn has_registration() -> Result<bool> {
    let has_registration = wallet().read().await.has_registration();
    Ok(has_registration)
}

#[async_runtime]
pub async fn register(pin: String) -> Result<()> {
    wallet().write().await.register(pin).await?;

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
