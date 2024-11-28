pub mod attested_key;
pub mod hw_keystore;
pub mod utils;

use std::panic::UnwindSafe;
use std::panic::{self};
use std::process;

macro_rules! log_panic {
    ($($arg:tt)*) => {
        cfg_if::cfg_if!  {
            if #[cfg(target_os = "android")] {
                log::error!($($arg)*);
            } else {
                eprintln!($($arg)*)
            }
        }
    }
}

fn print_panic<F>(task: F)
where
    F: FnOnce() + UnwindSafe,
{
    if let Err(error) = panic::catch_unwind(task) {
        if let Some(panic_message) = error.downcast_ref::<&str>() {
            log_panic!("Rust panic: {}", panic_message);
        } else if let Some(panic_message) = error.downcast_ref::<String>() {
            log_panic!("Rust panic: {}", panic_message);
        } else {
            log_panic!("Rust panic of unknown type occurred");
        }

        process::abort();
    }
}
