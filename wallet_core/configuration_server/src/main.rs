use std::error::Error;

use crate::settings::Settings;

mod server;
mod settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    server::serve(settings).await
}
