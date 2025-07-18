use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use cryptoki::context::CInitializeArgs;
use cryptoki::context::Pkcs11;
use cryptoki::mechanism::Mechanism;
use cryptoki::mechanism::aead::GcmParams;
use cryptoki::object::Attribute;
use cryptoki::object::AttributeType;
use cryptoki::object::KeyType;
use cryptoki::object::ObjectClass;
use cryptoki::object::ObjectHandle;
use cryptoki::types::AuthPin;
use der::Decode;
use der::Encode;
use der::asn1::OctetString;
use futures::future;
use p256::NistP256;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::AssociatedOid;
use r2d2_cryptoki::Pool;
use r2d2_cryptoki::SessionManager;
use r2d2_cryptoki::SessionType;
use r2d2_cryptoki::r2d2::LoggingErrorHandler;
use sec1::EcParameters;

use crypto::p256_der::verifying_key_sha256;
use crypto::utils::sha256;
use utils::spawn;

use crate::model::Hsm;
use crate::model::encrypted::Encrypted;
use crate::model::encrypted::InitializationVector;
use crate::model::wrapped_key::WrappedKey;

#[derive(Debug, thiserror::Error)]
pub enum HsmError {
    #[error("pkcs11 error: {0}")]
    Pkcs11(#[from] cryptoki::error::Error),

    #[error("r2d2 error: {0}")]
    R2d2(#[from] r2d2_cryptoki::r2d2::Error),

    #[error("sec1 error: {0}")]
    Sec1(#[from] sec1::der::Error),

    #[error("no initialized slot available")]
    NoInitializedSlotAvailable,

    #[error("p256 error: {0}")]
    P256(#[from] p256::ecdsa::Error),

    #[error("attribute not found: '{0}'")]
    AttributeNotFound(String),

    #[error("key not found: '{0}'")]
    KeyNotFound(String),

    #[cfg(feature = "mock")]
    #[error("hmac error: {0}")]
    Hmac(#[from] hmac::digest::MacError),
}

type Result<T> = std::result::Result<T, HsmError>;

pub struct PrivateKeyHandle(ObjectHandle);
pub struct PublicKeyHandle(ObjectHandle);

const AES_AUTHENTICATION_TAG_BITS: u64 = 128;

enum HandleType {
    Public,
    Private,
}

pub enum SigningMechanism {
    Ecdsa256,
    Sha256Hmac,
}

pub trait Pkcs11Client {
    async fn generate_aes_encryption_key(&self, identifier: &str) -> Result<PrivateKeyHandle>;
    async fn generate_generic_secret_key(&self, identifier: &str) -> Result<PrivateKeyHandle>;
    async fn generate_session_signing_key_pair(&self) -> Result<(PublicKeyHandle, PrivateKeyHandle)>;
    async fn generate_signing_key_pair(&self, identifier: &str) -> Result<(PublicKeyHandle, PrivateKeyHandle)>;
    async fn get_private_key_handle(&self, identifier: &str) -> Result<PrivateKeyHandle>;
    async fn get_public_key_handle(&self, identifier: &str) -> Result<PublicKeyHandle>;
    async fn get_verifying_key(&self, public_key_handle: PublicKeyHandle) -> Result<VerifyingKey>;
    async fn delete_key(&self, private_key_handle: PrivateKeyHandle) -> Result<()>;
    async fn sign(
        &self,
        private_key_handle: PrivateKeyHandle,
        mechanism: SigningMechanism,
        data: Arc<Vec<u8>>,
    ) -> Result<Vec<u8>>;
    async fn verify(
        &self,
        private_key_handle: PrivateKeyHandle,
        mechanism: SigningMechanism,
        data: Arc<Vec<u8>>,
        signature: Vec<u8>,
    ) -> Result<()>;
    async fn random_bytes(&self, length: u32) -> Result<Vec<u8>>;
    async fn encrypt(
        &self,
        key_handle: PrivateKeyHandle,
        iv: InitializationVector,
        data: Vec<u8>,
    ) -> Result<(Vec<u8>, InitializationVector)>;
    async fn decrypt(
        &self,
        key_handle: PrivateKeyHandle,
        iv: InitializationVector,
        encrypted_data: Vec<u8>,
    ) -> Result<Vec<u8>>;
    async fn wrap_key(
        &self,
        wrapping_key: PrivateKeyHandle,
        key: PrivateKeyHandle,
        public_key: VerifyingKey,
    ) -> Result<WrappedKey>;
    async fn unwrap_signing_key(
        &self,
        unwrapping_key: PrivateKeyHandle,
        wrapped_key: WrappedKey,
    ) -> Result<PrivateKeyHandle>;
    async fn generate_wrapped_key(&self, wrapping_key_identifier: &str) -> Result<(VerifyingKey, WrappedKey)>;
    async fn generate_wrapped_keys(
        &self,
        wrapping_key_identifier: &str,
        count: u64,
    ) -> Result<Vec<(String, VerifyingKey, WrappedKey)>> {
        future::try_join_all((0..count).map(|_| async move {
            let result = self.generate_wrapped_key(wrapping_key_identifier).await;
            result.map(|(pub_key, wrapped)| (verifying_key_sha256(&pub_key), pub_key, wrapped))
        }))
        .await
    }
    async fn sign_wrapped(
        &self,
        wrapping_key_identifier: &str,
        wrapped_key: WrappedKey,
        data: Arc<Vec<u8>>,
    ) -> Result<Signature>;
}

#[derive(Clone)]
pub struct Pkcs11Hsm {
    pool: Pool,
}

impl Pkcs11Hsm {
    pub fn new(
        library_path: PathBuf,
        user_pin: String,
        max_sessions: u8,
        max_session_lifetime: Duration,
    ) -> Result<Self> {
        let pkcs11_client = Pkcs11::new(library_path)?;
        pkcs11_client.initialize(CInitializeArgs::OsThreads)?;

        let slot = *pkcs11_client
            .get_slots_with_initialized_token()?
            .first()
            .ok_or(HsmError::NoInitializedSlotAvailable)?;

        let manager = SessionManager::new(pkcs11_client, slot, SessionType::RwUser(AuthPin::new(user_pin)));

        let pool = Pool::builder()
            .max_size(max_sessions.into())
            .max_lifetime(Some(max_session_lifetime))
            // This makes a pkcs11 call every time a connection is check out of the pool and should be evaluated in a
            // future performance test.
            .test_on_check_out(true)
            .error_handler(Box::new(LoggingErrorHandler))
            .build(manager)?;

        Ok(Self { pool })
    }

    #[cfg(feature = "settings")]
    pub fn from_settings(settings: crate::settings::Hsm) -> Result<Self> {
        Pkcs11Hsm::new(
            settings.library_path,
            settings.user_pin,
            settings.max_sessions,
            settings.max_session_lifetime,
        )
    }

    async fn get_key_handle(&self, identifier: &str, handle_type: HandleType) -> Result<ObjectHandle> {
        let pool = self.pool.clone();
        let identifier = String::from(identifier);

        spawn::blocking(move || {
            let session = pool.get()?;
            let object_handles = session.find_objects(&[
                Attribute::Private(matches!(handle_type, HandleType::Private)),
                Attribute::Label(identifier.clone().into()),
            ])?;
            let object_handle = object_handles
                .first()
                .copied()
                .ok_or(HsmError::KeyNotFound(identifier))?;
            Ok(object_handle)
        })
        .await
    }
}

impl Hsm for Pkcs11Hsm {
    type Error = HsmError;

    async fn generate_generic_secret_key(&self, identifier: &str) -> std::result::Result<(), Self::Error> {
        Pkcs11Client::generate_generic_secret_key(self, identifier)
            .await
            .map(|_| ())
    }

    async fn generate_aes_encryption_key(&self, identifier: &str) -> std::result::Result<(), Self::Error> {
        Pkcs11Client::generate_aes_encryption_key(self, identifier)
            .await
            .map(|_| ())
    }

    async fn generate_signing_key_pair(&self, identifier: &str) -> std::result::Result<(), Self::Error> {
        Pkcs11Client::generate_signing_key_pair(self, identifier)
            .await
            .map(|_| ())
    }

    async fn get_verifying_key(&self, identifier: &str) -> Result<VerifyingKey> {
        let handle = self.get_public_key_handle(identifier).await?;
        Pkcs11Client::get_verifying_key(self, handle).await
    }

    async fn delete_key(&self, identifier: &str) -> Result<()> {
        let handle = self.get_private_key_handle(identifier).await?;
        Pkcs11Client::delete_key(self, handle).await?;
        Ok(())
    }

    async fn sign_ecdsa(&self, identifier: &str, data: Arc<Vec<u8>>) -> std::result::Result<Signature, Self::Error> {
        let handle = self.get_private_key_handle(identifier).await?;
        let signature = Pkcs11Client::sign(self, handle, SigningMechanism::Ecdsa256, data).await?;
        Ok(Signature::from_slice(&signature)?)
    }

    async fn sign_hmac(&self, identifier: &str, data: Arc<Vec<u8>>) -> std::result::Result<Vec<u8>, Self::Error> {
        let handle = self.get_private_key_handle(identifier).await?;
        Pkcs11Client::sign(self, handle, SigningMechanism::Sha256Hmac, data).await
    }

    async fn verify_hmac(
        &self,
        identifier: &str,
        data: Arc<Vec<u8>>,
        signature: Vec<u8>,
    ) -> std::result::Result<(), Self::Error> {
        let handle = self.get_private_key_handle(identifier).await?;
        Pkcs11Client::verify(self, handle, SigningMechanism::Sha256Hmac, data, signature).await
    }

    async fn encrypt<T>(&self, identifier: &str, data: Vec<u8>) -> Result<Encrypted<T>> {
        let iv = self.random_bytes(32).await?;
        let handle = self.get_private_key_handle(identifier).await?;
        let (encrypted_data, initializiation_vector) =
            Pkcs11Client::encrypt(self, handle, InitializationVector(iv), data).await?;
        Ok(Encrypted::new(encrypted_data, initializiation_vector))
    }

    async fn decrypt<T>(&self, identifier: &str, encrypted: Encrypted<T>) -> Result<Vec<u8>> {
        let handle = self.get_private_key_handle(identifier).await?;
        Pkcs11Client::decrypt(self, handle, encrypted.iv, encrypted.data).await
    }
}

impl Pkcs11Client for Pkcs11Hsm {
    async fn generate_generic_secret_key(&self, identifier: &str) -> Result<PrivateKeyHandle> {
        let pool = self.pool.clone();
        let identifier = String::from(identifier);

        spawn::blocking(move || {
            let session = pool.get()?;

            let priv_key_template = &[
                Attribute::Token(true),
                Attribute::Private(true),
                Attribute::Sensitive(true),
                Attribute::Sign(true),
                Attribute::Class(ObjectClass::SECRET_KEY),
                Attribute::KeyType(KeyType::GENERIC_SECRET),
                Attribute::ValueLen(32.into()),
                Attribute::Label(identifier.clone().into()),
            ];

            let private_handle = session.generate_key(&Mechanism::GenericSecretKeyGen, priv_key_template)?;

            Ok(PrivateKeyHandle(private_handle))
        })
        .await
    }

    async fn generate_aes_encryption_key(&self, identifier: &str) -> Result<PrivateKeyHandle> {
        let pool = self.pool.clone();
        let identifier = String::from(identifier);

        spawn::blocking(move || {
            let session = pool.get()?;

            let priv_key_template = &[
                Attribute::Token(true),
                Attribute::Private(true),
                Attribute::Sensitive(true),
                Attribute::Encrypt(true),
                Attribute::Class(ObjectClass::SECRET_KEY),
                Attribute::KeyType(KeyType::AES),
                Attribute::ValueLen(32.into()),
                Attribute::Label(identifier.clone().into()),
            ];

            let private_handle = session.generate_key(&Mechanism::AesKeyGen, priv_key_template)?;

            Ok(PrivateKeyHandle(private_handle))
        })
        .await
    }

    async fn generate_session_signing_key_pair(&self) -> Result<(PublicKeyHandle, PrivateKeyHandle)> {
        let pool = self.pool.clone();

        spawn::blocking(move || {
            let session = pool.get()?;

            let mut oid = vec![];
            EcParameters::NamedCurve(NistP256::OID).encode_to_vec(&mut oid)?;

            let pub_key_template = &[Attribute::EcParams(oid)];
            let priv_key_template = &[
                Attribute::Token(false),
                Attribute::Private(true),
                Attribute::Extractable(true),
                Attribute::Derive(false),
                Attribute::Sign(false),
            ];

            let (public_handle, private_handle) =
                session.generate_key_pair(&Mechanism::EccKeyPairGen, pub_key_template, priv_key_template)?;

            Ok((PublicKeyHandle(public_handle), PrivateKeyHandle(private_handle)))
        })
        .await
    }

    async fn generate_signing_key_pair(&self, identifier: &str) -> Result<(PublicKeyHandle, PrivateKeyHandle)> {
        let pool = self.pool.clone();
        let identifier = String::from(identifier);

        spawn::blocking(move || {
            let session = pool.get()?;

            let mut oid = vec![];
            EcParameters::NamedCurve(NistP256::OID).encode_to_vec(&mut oid)?;

            let pub_key_template = &[Attribute::EcParams(oid), Attribute::Label(identifier.clone().into())];
            let priv_key_template = &[
                Attribute::Token(true),
                Attribute::Private(true),
                Attribute::Sensitive(true),
                Attribute::Extractable(false),
                Attribute::Derive(false),
                Attribute::Sign(true),
                Attribute::Label(identifier.into()),
            ];

            let (public_handle, private_handle) =
                session.generate_key_pair(&Mechanism::EccKeyPairGen, pub_key_template, priv_key_template)?;

            Ok((PublicKeyHandle(public_handle), PrivateKeyHandle(private_handle)))
        })
        .await
    }

    async fn get_private_key_handle(&self, identifier: &str) -> Result<PrivateKeyHandle> {
        self.get_key_handle(identifier, HandleType::Private)
            .await
            .map(PrivateKeyHandle)
    }

    async fn get_public_key_handle(&self, identifier: &str) -> Result<PublicKeyHandle> {
        self.get_key_handle(identifier, HandleType::Public)
            .await
            .map(PublicKeyHandle)
    }

    async fn get_verifying_key(&self, public_key_handle: PublicKeyHandle) -> Result<VerifyingKey> {
        let pool = self.pool.clone();

        spawn::blocking(move || {
            let session = pool.get()?;
            let attr = session
                .get_attributes(public_key_handle.0, &[AttributeType::EcPoint])?
                .first()
                .cloned()
                .ok_or(HsmError::AttributeNotFound(AttributeType::EcPoint.to_string()))?;

            match attr {
                Attribute::EcPoint(ec_point) => {
                    let octet_string = OctetString::from_der(&ec_point)?;
                    let public_key = VerifyingKey::from_sec1_bytes(octet_string.as_bytes())?;
                    Ok(public_key)
                }
                _ => Err(HsmError::AttributeNotFound(AttributeType::EcPoint.to_string())),
            }
        })
        .await
    }

    async fn delete_key(&self, private_key_handle: PrivateKeyHandle) -> Result<()> {
        let pool = self.pool.clone();

        spawn::blocking(move || {
            let session = pool.get()?;
            session.destroy_object(private_key_handle.0)?;
            Ok(())
        })
        .await
    }

    async fn sign(
        &self,
        private_key_handle: PrivateKeyHandle,
        mechanism: SigningMechanism,
        data: Arc<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        let pool = self.pool.clone();

        spawn::blocking(move || {
            let mechanism = match mechanism {
                SigningMechanism::Ecdsa256 => Mechanism::Ecdsa,
                SigningMechanism::Sha256Hmac => Mechanism::Sha256Hmac,
            };

            let session = pool.get()?;
            let signature = session.sign(&mechanism, private_key_handle.0, &sha256(&data))?;
            Ok(signature)
        })
        .await
    }

    async fn verify(
        &self,
        private_key_handle: PrivateKeyHandle,
        mechanism: SigningMechanism,
        data: Arc<Vec<u8>>,
        signature: Vec<u8>,
    ) -> Result<()> {
        let pool = self.pool.clone();

        spawn::blocking(move || {
            let mechanism = match mechanism {
                SigningMechanism::Ecdsa256 => Mechanism::Ecdsa,
                SigningMechanism::Sha256Hmac => Mechanism::Sha256Hmac,
            };

            let session = pool.get()?;
            session.verify(&mechanism, private_key_handle.0, &sha256(&data), &signature)?;

            Ok(())
        })
        .await
    }

    async fn random_bytes(&self, length: u32) -> Result<Vec<u8>> {
        let pool = self.pool.clone();

        spawn::blocking(move || {
            let session = pool.get()?;
            let data = session.generate_random_vec(length)?;
            Ok(data)
        })
        .await
    }

    async fn encrypt(
        &self,
        key_handle: PrivateKeyHandle,
        iv: InitializationVector,
        data: Vec<u8>,
    ) -> Result<(Vec<u8>, InitializationVector)> {
        let pool = self.pool.clone();

        spawn::blocking(move || {
            let session = pool.get()?;
            let gcm_params = GcmParams::new(&iv.0, &[], AES_AUTHENTICATION_TAG_BITS.into());
            let encrypted_data = session.encrypt(&Mechanism::AesGcm(gcm_params), key_handle.0, &data)?;
            Ok((encrypted_data, iv))
        })
        .await
    }

    async fn decrypt(
        &self,
        key_handle: PrivateKeyHandle,
        iv: InitializationVector,
        encrypted_data: Vec<u8>,
    ) -> Result<Vec<u8>> {
        let pool = self.pool.clone();

        spawn::blocking(move || {
            let session = pool.get()?;
            let gcm_params = GcmParams::new(&iv.0, &[], AES_AUTHENTICATION_TAG_BITS.into());
            let data = session.decrypt(&Mechanism::AesGcm(gcm_params), key_handle.0, &encrypted_data)?;
            Ok(data)
        })
        .await
    }

    async fn wrap_key(
        &self,
        wrapping_key: PrivateKeyHandle,
        key: PrivateKeyHandle,
        public_key: VerifyingKey,
    ) -> Result<WrappedKey> {
        let pool = self.pool.clone();

        spawn::blocking(move || {
            let session = pool.get()?;
            let wrapped_key_bytes = session.wrap_key(&Mechanism::AesKeyWrapPad, wrapping_key.0, key.0)?;
            Ok(WrappedKey::new(wrapped_key_bytes, public_key))
        })
        .await
    }

    async fn unwrap_signing_key(
        &self,
        unwrapping_key: PrivateKeyHandle,
        wrapped_key: WrappedKey,
    ) -> Result<PrivateKeyHandle> {
        let pool = self.pool.clone();

        spawn::blocking(move || {
            let session = pool.get()?;

            let result = session.unwrap_key(
                &Mechanism::AesKeyWrapPad,
                unwrapping_key.0,
                wrapped_key.wrapped_private_key(),
                &[
                    Attribute::KeyType(KeyType::EC),
                    Attribute::Token(false),
                    Attribute::Private(true),
                    Attribute::Class(ObjectClass::PRIVATE_KEY),
                ],
            )?;
            Ok(result)
        })
        .await
        .map(PrivateKeyHandle)
    }

    async fn generate_wrapped_key(&self, wrapping_key_identifier: &str) -> Result<(VerifyingKey, WrappedKey)> {
        let private_wrapping_handle = self.get_private_key_handle(wrapping_key_identifier).await?;
        let (public_handle, private_handle) = self.generate_session_signing_key_pair().await?;
        let verifying_key = Pkcs11Client::get_verifying_key(self, public_handle).await?;

        let wrapped = self
            .wrap_key(private_wrapping_handle, private_handle, verifying_key)
            .await?;

        Ok((verifying_key, wrapped))
    }

    async fn sign_wrapped(
        &self,
        wrapping_key_identifier: &str,
        wrapped_key: WrappedKey,
        data: Arc<Vec<u8>>,
    ) -> Result<Signature> {
        let private_wrapping_handle = self.get_private_key_handle(wrapping_key_identifier).await?;
        let private_handle = self.unwrap_signing_key(private_wrapping_handle, wrapped_key).await?;
        let signature = Pkcs11Client::sign(self, private_handle, SigningMechanism::Ecdsa256, data).await?;
        Ok(Signature::from_slice(&signature)?)
    }
}
