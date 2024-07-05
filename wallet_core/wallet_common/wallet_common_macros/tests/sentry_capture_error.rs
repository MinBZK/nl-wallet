use sentry::{test::with_captured_events, Level};
use thiserror::Error;

use wallet_common::error_category::{sentry_capture_error, ErrorCategory};

struct Wallet;

#[derive(ErrorCategory, Debug, Error)]
#[allow(dead_code)]
enum Error {
    #[error("Just some error")]
    #[category(critical)]
    CriticalError,
}

#[sentry_capture_error]
impl Wallet {
    pub fn do_something(&self) -> Result<(), Error> {
        Err(Error::CriticalError)
    }
}

trait Foo {
    fn foo(&self) -> Result<(), Error>;
}

#[sentry_capture_error]
impl Foo for Wallet {
    fn foo(&self) -> Result<(), Error> {
        Err(Error::CriticalError)
    }
}

#[sentry_capture_error]
fn bar() -> Result<(), Error> {
    Err(Error::CriticalError)
}

#[test]
fn test_do_something() {
    let wallet = Wallet;
    let events = with_captured_events(|| {
        let _ = wallet.do_something();
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(events[0].exception.values[0].ty, "CriticalError".to_string());
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn test_foo() {
    let wallet = Wallet;

    let events = with_captured_events(|| {
        let _ = wallet.foo();
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(events[0].exception.values[0].ty, "CriticalError".to_string());
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn test_bar() {
    let events = with_captured_events(|| {
        let _ = bar();
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(events[0].exception.values[0].ty, "CriticalError".to_string());
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn sentry_capture_error() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/sentry_capture_error/fail_*.rs");
}
