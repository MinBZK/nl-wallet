use anyhow::{Ok, Result};
use once_cell::sync::OnceCell;
use tokio::sync::{Mutex, MutexGuard};

use macros::async_runtime;
use wallet::{init_wallet, pin::validation::validate_pin, Wallet};

use crate::{async_runtime::get_or_try_init_async, models::pin::PinValidationResult};

static WALLET: OnceCell<Mutex<Wallet>> = OnceCell::new();

async fn lock_wallet() -> MutexGuard<'static, Wallet> {
    WALLET
        .get()
        .expect("Wallet must be initialized. Please execute `init()` first.")
        .lock()
        .await
}

pub fn init() -> Result<bool> {
    let runtime = get_or_try_init_async()?;
    let mut has_registration: Option<bool> = None;

    _ = WALLET.get_or_try_init(|| {
        runtime.block_on(async {
            let mut wallet = init_wallet();
            has_registration.replace(wallet.load_registration().await?);

            Ok(Mutex::new(wallet))
        })
    })?;

    Ok(has_registration.expect("Wallet may only be initialized once."))
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
