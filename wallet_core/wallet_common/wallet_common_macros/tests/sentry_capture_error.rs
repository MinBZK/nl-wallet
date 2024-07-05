use thiserror::Error;

use wallet_common::error_category::{sentry_capture_error, ErrorCategory};

struct Wallet;

#[derive(ErrorCategory, Debug, Error)]
#[allow(dead_code)]
enum Error {
    #[error("Just some error")]
    #[category(expected)]
    Unit,
}

#[sentry_capture_error]
impl Wallet {
    pub fn do_something(&self) -> Result<(), Error> {
        Err(Error::Unit)
    }
}

trait Foo {
    fn foo(&self) -> Result<(), Error>;
}

#[sentry_capture_error]
impl Foo for Wallet {
    fn foo(&self) -> Result<(), Error> {
        Err(Error::Unit)
    }
}

#[sentry_capture_error]
fn bar() -> Result<(), Error> {
    Err(Error::Unit)
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

#[test]
fn test_foo() {
    let wallet = Wallet;
    match wallet.foo() {
        Ok(_) => panic!("Expected error"),
        Err(error) => {
            dbg!(error);
        }
    }
}

#[test]
fn test_bar() {
    match bar() {
        Ok(_) => panic!("Expected error"),
        Err(error) => {
            dbg!(error);
        }
    }
}

#[test]
fn handle_error_category() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/handle_error_category/fail_*.rs");
}
