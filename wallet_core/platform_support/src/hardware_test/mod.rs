pub mod hw_keystore;
pub mod utils;

use std::{
    panic::{self, UnwindSafe},
    process,
};

fn print_panic<F>(task: F)
where
    F: FnOnce() + UnwindSafe,
{
    if let Err(error) = panic::catch_unwind(task) {
        if let Some(panic_message) = error.downcast_ref::<String>() {
            eprintln!("Rust panic: {}", panic_message);
        } else {
            eprintln!("Unknown Rust panic occurred");
        }

        process::abort();
    }
}
