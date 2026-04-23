use std::convert::Infallible;
use std::time::Duration;

use android_attest::attestation_extension::key_attestation::OsVersion;
use android_attest::attestation_extension::key_attestation::PatchLevel;
use apple_app_attest::AssertionCounter;
use async_dropper::AsyncDrop;
use async_dropper::AsyncDropper;
use async_trait::async_trait;
use chrono::Utc;
use crypto::utils::random_bytes;
use db_test::DbSetup;
use hsm::model::encrypted::Encrypted;
use hsm::model::encrypter::Encrypter;
use hsm::model::mock::MockPkcs11Client;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use url::Url;
use uuid::Uuid;
use wallet_provider_domain::model::wallet_user::AndroidHardwareIdentifiers;
use wallet_provider_domain::model::wallet_user::WalletId;
use wallet_provider_domain::model::wallet_user::WalletUserAttestationCreate;
use wallet_provider_domain::model::wallet_user::WalletUserCreate;

use crate::PersistenceConnection;
use crate::database::ConnectionOptions;
use crate::database::Db;
use crate::entity::wallet_flag;
use crate::entity::wallet_user_wua;
use crate::wallet_user::create_wallet_user;

#[derive(Debug, Clone, Copy)]
pub enum WalletDeviceVendor {
    Apple,
    Google,
}

pub async fn db_from_setup(db_setup: &DbSetup) -> Db {
    db_from_url(db_setup.wallet_provider_url()).await
}

async fn db_from_url(url: Url) -> Db {
    Db::new(
        url,
        ConnectionOptions {
            connect_timeout: Duration::from_secs(1),
            max_connections: 5,
        },
    )
    .await
    .expect("Could not connect to database")
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

pub async fn create_wallet_user_with_random_keys<S, T>(db: &T, vendor: WalletDeviceVendor, wallet_id: WalletId) -> Uuid
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
            identifiers: AndroidHardwareIdentifiers {
                brand: Some("Brand Name".to_string()),
                model: Some("Model Name".to_string()),
                os_version: Some(OsVersion {
                    major: 3,
                    minor: 2,
                    sub_minor: 1,
                }),
                os_patch_level: Some(PatchLevel { year: 2026, month: 1 }),
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

pub fn clear_flags_dropper(db_setup: &DbSetup) -> AsyncDropper<ClearFlags> {
    AsyncDropper::new(ClearFlags(Some(db_setup.wallet_provider_url())))
}

#[derive(Default)]
pub struct ClearFlags(Option<Url>);

#[async_trait]
impl AsyncDrop for ClearFlags {
    async fn async_drop(&mut self) {
        let db = db_from_url(self.0.as_ref().unwrap().clone()).await;
        if let Err(err) = wallet_flag::Entity::delete_many().exec(db.connection()).await {
            tracing::error!("Could not delete flags {err}");
        }
    }
}
