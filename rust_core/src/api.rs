use anyhow::Result;
use flutter_data_types::PinResult;
use once_cell::sync::Lazy;
use platform_support::hw_keystore::PlatformSigningKey;
use std::sync::Mutex;

use crate::{
    wallet::{pin::validate_pin, Wallet},
    wp::AccountServer,
};

#[cfg(any(target_os = "android", target_os = "ios"))]
type PlatformSigningKeyImpl = platform_support::hw_keystore::hardware::HardwareSigningKey;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
type PlatformSigningKeyImpl = platform_support::hw_keystore::software::SoftwareSigningKey;

const WALLET_KEY_ID: &str = "wallet";

static WALLET: Lazy<Mutex<Wallet<AccountServer, PlatformSigningKeyImpl>>> = Lazy::new(|| {
    let hw_privkey = PlatformSigningKeyImpl::signing_key(WALLET_KEY_ID)
        .expect("Could not fetch or generate wallet key");

    let account_server = AccountServer::new_stub(); // TODO
    let pubkey = account_server.pubkey.clone();

    Mutex::new(Wallet::new(account_server, pubkey, hw_privkey))
});

pub fn is_valid_pin(pin: String) -> Vec<u8> {
    let pin_result = PinResult::from(validate_pin(&pin));
    bincode::serialize(&pin_result).unwrap()
}

pub fn register(pin: String) -> Result<()> {
    // TODO return differentiated errors?
    WALLET.lock().expect("wallet lock failed").register(pin)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_is_valid_pin(pin: &str) -> bool {
        let serialized_pin_result = is_valid_pin(pin.to_owned());
        let pin_result = bincode::deserialize(&serialized_pin_result).unwrap();
        match pin_result {
            PinResult::Ok => true,
            PinResult::Err(_) => false,
        }
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
