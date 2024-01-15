use std::error::Error;

use configuration_server::read_config_jwt;

use crate::settings::Settings;

mod server;
mod settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;
    let config_jwt = read_config_jwt();

    server::serve(settings, config_jwt).await?;

    Ok(())
}
