use std::sync::Arc;

use p256::ecdsa::signature::Verifier;

use wallet_common::utils::{random_bytes, random_string};
use wallet_provider::settings::Settings;
use wallet_provider_service::hsm::{Hsm, Pkcs11Client};

#[tokio::test]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn generate_key_and_sign() {
    let settings = Settings::new().unwrap();
    let hsm = Hsm::new(settings.hsm.library_path, settings.hsm.user_pin).unwrap();

    let prefix = "wallet_user_1";
    let identifier = random_string(8);
    let public_key = hsm.generate_key(prefix, &identifier).await.unwrap();

    let data = Arc::new(random_bytes(32));
    let signature = hsm.sign(prefix, &identifier, Arc::clone(&data)).await.unwrap();
    public_key.verify(data.as_ref(), &signature).unwrap();

    hsm.delete_key(prefix, &identifier).await.unwrap();
}
