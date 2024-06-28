use thiserror::Error;

use wallet_common::error_category::{handle_error_category, ErrorCategory};

struct Wallet;

#[derive(ErrorCategory, Debug, Error)]
#[allow(dead_code)]
enum Error {
    #[error("Just some error")]
    #[category(expected)]
    Unit,
}

#[handle_error_category]
impl Wallet {
    pub fn do_something(&self) -> Result<(), Error> {
        Err(Error::Unit)
    }
}

#[test]
fn test_do_something() {
    let wallet = Wallet;
    match wallet.do_something() {
        Ok(_) => panic!("Expected error"),
        Err(error) => {
            dbg!(error);
        }
    }
}
