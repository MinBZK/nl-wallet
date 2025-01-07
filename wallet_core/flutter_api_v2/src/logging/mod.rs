mod tracing;

#[cfg(debug_assertions)]
mod panic;

#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "ios")]
mod ios;

use parking_lot::Once;

use self::tracing::init_tracing_subscriber;

static LOGGING: Once = Once::new();

pub fn init_logging() {
    // Make sure this initializer can be called multiple times, but executes only once.
    LOGGING.call_once(|| {
        // Set up a subscriber to log to the relevant output for the platform.
        init_tracing_subscriber();

        // Set a custom panic handler for debug builds. For release builds
        // we can rely on Flutter logging panics as uncaught exceptions.
        // As init_tracing_subscriber() might panic, we want that to be caught by
        // the default handler, as we cannot log anything yet anyway.
        #[cfg(debug_assertions)]
        self::panic::init_panic_logger();
    });
}
