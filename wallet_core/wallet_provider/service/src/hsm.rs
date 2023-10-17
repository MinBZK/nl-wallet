use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use cryptoki::{
    context::{CInitializeArgs, Pkcs11},
    mechanism::Mechanism,
    object::{Attribute, AttributeType, ObjectHandle},
    session::Session,
    types::AuthPin,
};
use der::{asn1::OctetString, Decode, Encode};
use futures::future;
use p256::{
    ecdsa::{Signature, VerifyingKey},
    pkcs8::AssociatedOid,
    NistP256,
};
use r2d2_cryptoki::{Pool, SessionManager, SessionType};
use sec1::EcParameters;

use wallet_common::{spawn, utils::sha256};

#[derive(Debug, thiserror::Error)]
pub enum HsmError {
    #[error("pkcs11 error: {0}")]
    Pkcs11(#[from] cryptoki::error::Error),

    #[error("r2d2 error: {0}")]
    R2d2(#[from] r2d2_cryptoki::r2d2::Error),

    #[error("sec1 error: {0}")]
    Sec1(#[from] sec1::der::Error),

    #[error("p256 error: {0}")]
    P256(#[from] p256::ecdsa::Error),

    #[error("no initialized slot available")]
    NoInitializedSlotAvailable,

    #[error("attribute not found: '{0}'")]
    AttributeNotFound(String),

    #[error("key not found: '{0}'")]
    KeyNotFound(String),
}

type Result<T> = std::result::Result<T, HsmError>;

#[async_trait]
pub trait Pkcs11Client {
    async fn generate_key(&self, key_prefix: &str, identifier: &str) -> Result<VerifyingKey>;

    async fn generate_keys(&self, key_prefix: &str, identifiers: &[&str]) -> Result<Vec<(String, VerifyingKey)>> {
        future::try_join_all(identifiers.iter().map(|identifier| async move {
            let result = self.generate_key(key_prefix, identifier).await;
            result.map(|pub_key| (String::from(*identifier), pub_key))
        }))
        .await
    }

    async fn delete_key(&self, key_prefix: &str, identifier: &str) -> Result<()>;

    async fn sign(&self, key_prefix: &str, identifier: &str, data: Arc<Vec<u8>>) -> Result<Signature>;

    async fn sign_multiple(
        &self,
        key_prefix: &str,
        identifiers: &[&str],
        data: Arc<Vec<u8>>,
    ) -> Result<Vec<(String, Signature)>> {
        future::try_join_all(identifiers.iter().map(|identifier| async {
            self.sign(key_prefix, identifier, Arc::clone(&data))
                .await
                .map(|signature| (String::from(*identifier), signature))
        }))
        .await
    }
}

pub struct Hsm {
    pool: Pool,
}

impl Hsm {
    pub fn new(library_path: PathBuf, user_pin: String) -> Result<Self> {
        let pkcs11_client = Pkcs11::new(library_path)?;
        pkcs11_client.initialize(CInitializeArgs::OsThreads)?;

        let slot = *pkcs11_client
            .get_slots_with_initialized_token()?
            .first()
            .ok_or(HsmError::NoInitializedSlotAvailable)?;

        let manager = SessionManager::new(pkcs11_client, slot, SessionType::RwUser(AuthPin::new(user_pin)));

        let pool = Pool::builder().build(manager).unwrap();
        Ok(Self { pool })
    }

    fn find_key_by_id(session: &Session, id: &str) -> Result<ObjectHandle> {
        let object_handles = session.find_objects(&[Attribute::Token(true), Attribute::Id(id.into())])?;
        let object_handle = object_handles
            .first()
            .cloned()
            .ok_or(HsmError::KeyNotFound(String::from(id)))?;
        Ok(object_handle)
    }

    fn key_identifier(prefix: &str, identifier: &str) -> String {
        format!("{prefix}_{identifier}")
    }
}

#[async_trait]
impl Pkcs11Client for Hsm {
    async fn generate_key(&self, key_prefix: &str, identifier: &str) -> Result<VerifyingKey> {
        let pool = self.pool.clone();
        let key_identifier = Hsm::key_identifier(key_prefix, identifier);

        spawn::blocking(move || {
            let session = pool.get()?;

            let mut oid = vec![];
            EcParameters::NamedCurve(NistP256::OID).encode_to_vec(&mut oid)?;

            let pub_key_template = &[Attribute::EcParams(oid)];
            let priv_key_template = &[
                Attribute::Token(true),
                Attribute::Private(true),
                Attribute::Sensitive(true),
                Attribute::Extractable(false),
                Attribute::Derive(false),
                Attribute::Sign(true),
                Attribute::Id(key_identifier.clone().into()),
            ];

            let (public_key_handle, _) =
                session.generate_key_pair(&Mechanism::EccKeyPairGen, pub_key_template, priv_key_template)?;

            let attr = session
                .get_attributes(public_key_handle, &[AttributeType::EcPoint])?
                .first()
                .cloned()
                .ok_or(HsmError::AttributeNotFound(AttributeType::EcPoint.to_string()))?;

            match attr {
                Attribute::EcPoint(ec_point) => {
                    let octet_string = OctetString::from_der(&ec_point)?;
                    let public_key = VerifyingKey::from_sec1_bytes(octet_string.as_bytes())?;
                    Ok(public_key)
                }
                _ => Err(HsmError::KeyNotFound(key_identifier)),
            }
        })
        .await
    }

    async fn delete_key(&self, key_prefix: &str, identifier: &str) -> Result<()> {
        let pool = self.pool.clone();
        let key_identifier = Hsm::key_identifier(key_prefix, identifier);

        spawn::blocking(move || {
            let session = pool.get()?;
            let key_handle = Hsm::find_key_by_id(&session, &key_identifier)?;
            session.destroy_object(key_handle)?;
            Ok(())
        })
        .await
    }

    async fn sign(&self, key_prefix: &str, identifier: &str, data: Arc<Vec<u8>>) -> Result<Signature> {
        let pool = self.pool.clone();
        let key_identifier = Hsm::key_identifier(key_prefix, identifier);

        spawn::blocking(move || {
            let session = pool.get()?;

            let object_handles =
                session.find_objects(&[Attribute::Token(true), Attribute::Id(key_identifier.clone().into())])?;

            let private_key_handle = object_handles
                .first()
                .cloned()
                .ok_or(HsmError::KeyNotFound(key_identifier))?;

            let signature = session.sign(&Mechanism::Ecdsa, private_key_handle, &sha256(data.as_ref()))?;
            Ok(Signature::from_slice(&signature)?)
        })
        .await
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use std::sync::Arc;

    use async_trait::async_trait;
    use dashmap::DashMap;
    use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
    use rand::rngs::OsRng;

    use crate::hsm::{Hsm, Pkcs11Client, Result};

    pub struct MockPkcs11Client(DashMap<String, SigningKey>);

    impl MockPkcs11Client {
        pub fn get_key(&self, key_prefix: &str, identifier: &str) -> Result<SigningKey> {
            let entry = self.0.get(&Hsm::key_identifier(key_prefix, identifier)).unwrap();
            let key = entry.value().clone();
            Ok(key)
        }
    }

    impl Default for MockPkcs11Client {
        fn default() -> Self {
            Self(DashMap::new())
        }
    }

    #[async_trait]
    impl Pkcs11Client for MockPkcs11Client {
        async fn generate_key(&self, key_prefix: &str, identifier: &str) -> Result<VerifyingKey> {
            let key = SigningKey::random(&mut OsRng);
            let verifying_key = *key.verifying_key();
            self.0.insert(Hsm::key_identifier(key_prefix, identifier), key);
            Ok(verifying_key)
        }

        async fn delete_key(&self, key_prefix: &str, identifier: &str) -> Result<()> {
            self.0.remove(&Hsm::key_identifier(key_prefix, identifier));
            Ok(())
        }

        async fn sign(&self, key_prefix: &str, identifier: &str, data: Arc<Vec<u8>>) -> Result<Signature> {
            let entry = self.0.get(&Hsm::key_identifier(key_prefix, identifier)).unwrap();
            let key = entry.value();
            let signature = Signer::sign(key, data.as_ref());
            Ok(signature)
        }
    }
}
