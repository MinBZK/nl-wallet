use sentry::{test::with_captured_events, Level};

// Macro is applied in module, to verify whether `pub` is handled correctly
mod my_module {
    use thiserror::Error;

    use error_category::{sentry_capture_error, ErrorCategory};

    // Test `sentry_capture_error` on regular functions
    #[sentry_capture_error]
    pub fn foo() -> Result<(), Error> {
        Err(Error::CriticalError)
    }

    #[sentry_capture_error]
    pub fn bar<'a, T>(_value: &'a T) -> Result<&'a str, Error> {
        Err(Error::CriticalError)
    }

    #[sentry_capture_error]
    pub async fn baz<'a, T>(_value: &'a T) -> Result<&'a str, Error> {
        Err(Error::CriticalError)
    }

    pub struct Wallet;

    #[derive(ErrorCategory, Debug, Error)]
    pub enum Error {
        #[error("Just some error")]
        #[category(critical)]
        CriticalError,
    }

    // Test `sentry_capture_error` on functions in `impl` blocks
    impl Wallet {
        #[sentry_capture_error]
        pub fn test_method(&self) -> Result<(), Error> {
            Err(Error::CriticalError)
        }

        #[sentry_capture_error]
        pub(crate) fn test_associated_fn<'a, T>(_value: &'a T) -> Result<&'a str, Error> {
            Err(Error::CriticalError)
        }
    }

    pub trait Foo {
        fn foo(&self) -> Result<(), Error>;
        fn bar<'a, T>(&self, value: &'a T) -> Result<&'a str, Error>;
        async fn baz<'a, T>(&self, _value: &'a T) -> Result<&'a str, Error>;
    }

    // Test `sentry_capture_error` on all functions in `impl Trait` blocks
    #[sentry_capture_error]
    impl Foo for Wallet {
        fn foo(&self) -> Result<(), Error> {
            Err(Error::CriticalError)
        }

        fn bar<'a, T>(&self, _value: &'a T) -> Result<&'a str, Error> {
            Err(Error::CriticalError)
        }

        async fn baz<'a, T>(&self, _value: &'a T) -> Result<&'a str, Error> {
            Err(Error::CriticalError)
        }
    }

    pub struct Purse<T>(pub T);

    pub trait MoneyContainer<T> {
        fn foo(&self) -> Result<T, Error>;
        fn bar<'a>(&self, value: &'a T) -> Result<&'a str, Error>;
        async fn baz<'a>(&self, _value: &'a T) -> Result<&'a str, Error>;
    }

    // Test `sentry_capture_error` on all functions in `impl Trait` blocks with generics
    #[sentry_capture_error]
    impl<T> MoneyContainer<T> for Purse<T> {
        fn foo(&self) -> Result<T, Error> {
            Err(Error::CriticalError)
        }

        fn bar<'a>(&self, _value: &'a T) -> Result<&'a str, Error> {
            Err(Error::CriticalError)
        }

        async fn baz<'a>(&self, _value: &'a T) -> Result<&'a str, Error> {
            Err(Error::CriticalError)
        }
    }
}

use my_module::*;

#[test]
fn test_foo() {
    let events = with_captured_events(|| {
        let _ = foo();
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(
        events[0].exception.values[0].ty,
        "sentry_capture_error::my_module::Error::CriticalError".to_string()
    );
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn test_bar() {
    let events = with_captured_events(|| {
        let _ = bar(&42);
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(
        events[0].exception.values[0].ty,
        "sentry_capture_error::my_module::Error::CriticalError".to_string()
    );
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn test_baz() {
    let events = with_captured_events(|| {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let _ = baz(&42).await;
        });
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(
        events[0].exception.values[0].ty,
        "sentry_capture_error::my_module::Error::CriticalError".to_string()
    );
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn test_test_method() {
    let wallet = Wallet;
    let events = with_captured_events(|| {
        let _ = wallet.test_method();
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(
        events[0].exception.values[0].ty,
        "sentry_capture_error::my_module::Error::CriticalError".to_string()
    );
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn test_test_associated_fn() {
    let events = with_captured_events(|| {
        let _ = Wallet::test_associated_fn(&42);
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(
        events[0].exception.values[0].ty,
        "sentry_capture_error::my_module::Error::CriticalError".to_string()
    );
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn test_wallet_foo() {
    let wallet = Wallet;

    let events = with_captured_events(|| {
        let _ = wallet.foo();
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(
        events[0].exception.values[0].ty,
        "sentry_capture_error::my_module::Error::CriticalError".to_string()
    );
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn test_wallet_bar() {
    let wallet = Wallet;
    let events = with_captured_events(|| {
        let _ = wallet.bar(&42);
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(
        events[0].exception.values[0].ty,
        "sentry_capture_error::my_module::Error::CriticalError".to_string()
    );
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn test_wallet_baz() {
    let wallet = Wallet;
    let events = with_captured_events(|| {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let _ = wallet.baz(&42).await;
        });
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(
        events[0].exception.values[0].ty,
        "sentry_capture_error::my_module::Error::CriticalError".to_string()
    );
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn test_purse_foo() {
    let purse = Purse(42);

    let events = with_captured_events(|| {
        let _ = purse.foo();
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(
        events[0].exception.values[0].ty,
        "sentry_capture_error::my_module::Error::CriticalError".to_string()
    );
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn test_purse_bar() {
    let purse = Purse(42);

    let events = with_captured_events(|| {
        let _ = purse.bar(&42);
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(
        events[0].exception.values[0].ty,
        "sentry_capture_error::my_module::Error::CriticalError".to_string()
    );
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn test_purse_baz() {
    let purse = Purse(42);

    let events = with_captured_events(|| {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let _ = purse.baz(&42).await;
        });
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(
        events[0].exception.values[0].ty,
        "sentry_capture_error::my_module::Error::CriticalError".to_string()
    );
    assert_eq!(events[0].exception.values[0].value, Some("Just some error".to_string()));
}

#[test]
fn sentry_capture_error() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/sentry_capture_error/fail_*.rs");
}
