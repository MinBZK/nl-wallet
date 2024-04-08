use std::error::Error;

use crate::settings::Settings;

mod gba;
mod haal_centraal;
mod server;
mod settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    haal_centraal::initialize_eager();

    server::serve(Settings::new()?).await?;
    Ok(())
}
