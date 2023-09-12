use serde::Serialize;
use std::error::Error;

use wallet_common::account::serialization::DerVerifyingKey;
use wallet_provider::settings::Settings;

#[derive(Serialize)]
struct VerifyingKeys {
    certificate_verifying_key: String,
    instruction_result_verifying_key: String,
}

impl From<Settings> for VerifyingKeys {
    fn from(settings: Settings) -> Self {
        Self {
            certificate_verifying_key: DerVerifyingKey::from(&settings.certificate_private_key).to_string(),
            instruction_result_verifying_key: DerVerifyingKey::from(&settings.instruction_result_private_key)
                .to_string(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let settings = Settings::new()?;
    let verifying_keys: VerifyingKeys = settings.into();
    println!("{}", serde_json::to_string(&verifying_keys)?);
    Ok(())
}
