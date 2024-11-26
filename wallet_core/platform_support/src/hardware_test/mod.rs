pub mod attested_key;
pub mod hw_keystore;
pub mod utils;

use std::panic::UnwindSafe;
use std::panic::{self};
use std::process;

fn print_panic<F>(task: F)
where
    F: FnOnce() + UnwindSafe,
{
    if let Err(error) = panic::catch_unwind(task) {
        if let Some(panic_message) = error.downcast_ref::<String>() {
            eprintln!("Rust panic: {}", panic_message);
        } else {
            eprintln!("Rust panic of unknown type occurred");
        }

        process::abort();
    }
}
