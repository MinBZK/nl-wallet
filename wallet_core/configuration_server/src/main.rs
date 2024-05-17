use std::error::Error;

use wallet_common::try_init_sentry;

use crate::settings::Settings;

mod server;
mod settings;

// Cannot use #[tokio::main], see: https://docs.sentry.io/platforms/rust/#async-main-function
fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let settings = Settings::new().unwrap();

    // Retain [`ClientInitGuard`]
    let _guard = try_init_sentry!(settings.sentry);

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async { server::serve(settings).await })
}
