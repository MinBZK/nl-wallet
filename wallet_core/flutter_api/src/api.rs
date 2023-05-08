use anyhow::{Ok, Result};
use once_cell::sync::OnceCell;
use tokio::sync::{Mutex, MutexGuard};

use macros::async_runtime;
use wallet::{init_wallet, pin::validation::validate_pin, Wallet};

use crate::{async_runtime::init_async_runtime, models::pin::PinValidationResult};

static WALLET: OnceCell<Mutex<Wallet>> = OnceCell::new();

async fn lock_wallet() -> MutexGuard<'static, Wallet> {
    WALLET
        .get()
        .expect("Wallet must be initialized. Please execute `init()` first.")
        .lock()
        .await
}

pub fn init() -> Result<bool> {
    // Initialize the async runtime so the #[async_runtime] macro can be used.
    // As creating the wallet below could fail and init() could be called again,
    // init_async_runtime() should not fail when being called more than once.
    init_async_runtime()?;

    let mut created_has_registration: Option<bool> = None;

    _ = WALLET.get_or_try_init(|| {
        // This closure will only be called if WALLET is currently empty.
        let (wallet, has_registration) = create_wallet()?;
        created_has_registration.replace(has_registration);

        Ok(wallet)
    })?;

    // Read created_has_registration, which is only None if the async block above
    // did not execute. This implies that init() was called successfully more than once.
    let has_registration = created_has_registration.expect("Wallet may only be initialized once.");

    Ok(has_registration)
}

#[async_runtime]
async fn create_wallet() -> Result<(Mutex<Wallet>, bool)> {
    let wallet = init_wallet().await?;
    let has_registration = wallet.has_registration();

    Ok((Mutex::new(wallet), has_registration))
}

pub fn is_valid_pin(pin: String) -> Vec<u8> {
    let pin_result = PinValidationResult::from(validate_pin(&pin));
    bincode::serialize(&pin_result).unwrap()
}

#[async_runtime]
pub async fn register(pin: String) -> Result<()> {
    // TODO return differentiated errors?
    lock_wallet().await.register(pin).await
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
