use std::convert::Infallible;

use chrono::Utc;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use uuid::Uuid;

use android_attest::attestation_extension::key_attestation::OsVersion;
use android_attest::attestation_extension::key_attestation::PatchLevel;
use apple_app_attest::AssertionCounter;
use crypto::utils::random_bytes;
use hsm::model::encrypted::Encrypted;
use hsm::model::encrypter::Encrypter;
use hsm::model::mock::MockPkcs11Client;
use wallet_provider_database_settings::Settings;
use wallet_provider_domain::model::wallet_user::AndroidAttestations;
use wallet_provider_domain::model::wallet_user::WalletUserAttestationCreate;
use wallet_provider_domain::model::wallet_user::WalletUserCreate;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::database::Db;
use crate::entity::wallet_user_wua;
use crate::wallet_user::create_wallet_user;

#[derive(Debug, Clone, Copy)]
pub enum WalletDeviceVendor {
    Apple,
    Google,
}

pub async fn db_from_env() -> Result<Db, PersistenceError> {
    let settings = Settings::new().unwrap();
    Db::new(settings.url, Default::default()).await
}

pub async fn encrypted_pin_key(identifier: &str) -> Encrypted<VerifyingKey> {
    Encrypter::<VerifyingKey>::encrypt(
        &MockPkcs11Client::<Infallible>::default(),
        identifier,
        *SigningKey::random(&mut OsRng).verifying_key(),
    )
    .await
    .unwrap()
}

pub async fn create_wallet_user_with_random_keys<S, T>(db: &T, vendor: WalletDeviceVendor, wallet_id: String) -> Uuid
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let attestation = match vendor {
        WalletDeviceVendor::Apple => WalletUserAttestationCreate::Apple {
            data: random_bytes(64),
            assertion_counter: AssertionCounter::default(),
        },
        WalletDeviceVendor::Google => WalletUserAttestationCreate::Android {
            certificate_chain: vec![random_bytes(64)],
            integrity_verdict_json: "{}".to_string(),
            attestations: AndroidAttestations {
                brand: Some("Brand Name".to_string()),
                model: Some("Model Name".to_string()),
                os_version: Some(OsVersion {
                    major: 3,
                    minor: 2,
                    sub_minor: 1,
                }),
                os_patch_level: Some(PatchLevel {
                    year: 2026,
                    month: 1,
                    day: None,
                }),
            },
        },
    };

    create_wallet_user(
        db,
        WalletUserCreate {
            wallet_id,
            hw_pubkey: *SigningKey::random(&mut OsRng).verifying_key(),
            encrypted_pin_pubkey: encrypted_pin_key("key1").await,
            attestation_date_time: Utc::now(),
            attestation,
            revocation_code_hmac: random_bytes(32),
        },
    )
    .await
    .expect("Could not create wallet user")
}

pub async fn truncate_wuas<S, T>(db: &T)
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user_wua::Entity::delete_many()
        .exec(db.connection())
        .await
        .expect("should delete all WUA ids");
}
