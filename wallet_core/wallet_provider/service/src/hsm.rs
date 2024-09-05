use std::{path::PathBuf, sync::Arc, time::Duration};

use cryptoki::{
    context::{CInitializeArgs, Pkcs11},
    mechanism::{aead::GcmParams, Mechanism},
    object::{Attribute, AttributeType, KeyType, ObjectClass, ObjectHandle},
    types::AuthPin,
};
use der::{asn1::OctetString, Decode, Encode};
use p256::{
    ecdsa::{Signature, VerifyingKey},
    pkcs8::AssociatedOid,
    NistP256,
};
use r2d2_cryptoki::{Pool, SessionManager, SessionType};
use sec1::EcParameters;

use wallet_common::{spawn, utils::sha256};
use wallet_provider_domain::model::{
    encrypted::{Encrypted, InitializationVector},
    encrypter::{Decrypter, Encrypter},
    hsm,
    hsm::{Hsm, WalletUserHsm},
    wallet_user::WalletId,
    wrapped_key::WrappedKey,
};

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

    #[cfg(any(test, feature = "mock"))]
    #[error("hmac error: {0}")]
    Hmac(#[from] hmac::digest::MacError),
}

type Result<T> = std::result::Result<T, HsmError>;

pub(crate) struct PrivateKeyHandle(ObjectHandle);
pub(crate) struct PublicKeyHandle(ObjectHandle);

const AES_AUTHENTICATION_TAG_BITS: u64 = 128;

enum HandleType {
    Public,
    Private,
}

pub(crate) enum SigningMechanism {
    Ecdsa256,
    Sha256Hmac,
}

pub(crate) trait Pkcs11Client {
    async fn generate_generic_secret_key(&self, identifier: &str) -> Result<PrivateKeyHandle>;
    async fn generate_session_signing_key_pair(&self) -> Result<(PublicKeyHandle, PrivateKeyHandle)>;
    async fn generate_signing_key_pair(&self, identifier: &str) -> Result<(PublicKeyHandle, PrivateKeyHandle)>;
    async fn get_private_key_handle(&self, identifier: &str) -> Result<PrivateKeyHandle>;
    async fn get_public_key_handle(&self, identifier: &str) -> Result<PublicKeyHandle>;
    async fn get_verifying_key(&self, public_key_handle: PublicKeyHandle) -> Result<VerifyingKey>;
    async fn wrap_key(&self, wrapping_key: PrivateKeyHandle, key: PrivateKeyHandle) -> Result<WrappedKey>;
    async fn unwrap_signing_key(
        &self,
        unwrapping_key: PrivateKeyHandle,
        wrapped_key: WrappedKey,
    ) -> Result<PrivateKeyHandle>;
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
}

#[derive(Clone)]
pub struct Pkcs11Hsm {
    pool: Pool,
    wrapping_key_identifier: String,
}

impl Pkcs11Hsm {
    pub fn new(
        library_path: PathBuf,
        user_pin: String,
        max_sessions: u8,
        max_session_lifetime: Duration,
        wrapping_key_identifier: String,
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
            .build(manager)
            .unwrap();

        Ok(Self {
            pool,
            wrapping_key_identifier,
        })
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
                .cloned()
                .ok_or(HsmError::KeyNotFound(identifier))?;
            Ok(object_handle)
        })
        .await
    }
}

impl Encrypter<VerifyingKey> for Pkcs11Hsm {
    type Error = HsmError;

    async fn encrypt(
        &self,
        key_identifier: &str,
        data: VerifyingKey,
    ) -> std::result::Result<Encrypted<VerifyingKey>, Self::Error> {
        let bytes: Vec<u8> = data.to_sec1_bytes().to_vec();
        Hsm::encrypt(self, key_identifier, bytes).await
    }
}

impl Decrypter<VerifyingKey> for Pkcs11Hsm {
    type Error = HsmError;

    async fn decrypt(
        &self,
        key_identifier: &str,
        encrypted: Encrypted<VerifyingKey>,
    ) -> std::result::Result<VerifyingKey, Self::Error> {
        let decrypted = Hsm::decrypt(self, key_identifier, encrypted).await?;
        Ok(VerifyingKey::from_sec1_bytes(&decrypted)?)
    }
}

impl WalletUserHsm for Pkcs11Hsm {
    type Error = HsmError;

    async fn generate_wrapped_key(&self) -> Result<(VerifyingKey, WrappedKey)> {
        let private_wrapping_handle = self.get_private_key_handle(&self.wrapping_key_identifier).await?;
        let (public_handle, private_handle) = self.generate_session_signing_key_pair().await?;

        let wrapped = self.wrap_key(private_wrapping_handle, private_handle).await?;
        let verifying_key = Pkcs11Client::get_verifying_key(self, public_handle).await?;
        Ok((verifying_key, wrapped))
    }

    async fn generate_key(&self, wallet_id: &WalletId, identifier: &str) -> Result<VerifyingKey> {
        let key_identifier = hsm::key_identifier(wallet_id, identifier);
        let (public_handle, _private_handle) = self.generate_signing_key_pair(&key_identifier).await?;
        Pkcs11Client::get_verifying_key(self, public_handle).await
    }

    async fn sign_wrapped(&self, wrapped_key: WrappedKey, data: Arc<Vec<u8>>) -> Result<Signature> {
        let private_wrapping_handle = self.get_private_key_handle(&self.wrapping_key_identifier).await?;
        let private_handle = self.unwrap_signing_key(private_wrapping_handle, wrapped_key).await?;
        let signature = Pkcs11Client::sign(self, private_handle, SigningMechanism::Ecdsa256, data).await?;
        Ok(Signature::from_slice(&signature)?)
    }

    async fn sign(&self, wallet_id: &WalletId, identifier: &str, data: Arc<Vec<u8>>) -> Result<Signature> {
        let key_identifier = hsm::key_identifier(wallet_id, identifier);
        let handle = self.get_private_key_handle(&key_identifier).await?;
        let signature = Pkcs11Client::sign(self, handle, SigningMechanism::Ecdsa256, data).await?;
        Ok(Signature::from_slice(&signature)?)
    }
}

impl Hsm for Pkcs11Hsm {
    type Error = HsmError;

    async fn generate_generic_secret_key(&self, identifier: &str) -> std::result::Result<(), Self::Error> {
        Pkcs11Client::generate_generic_secret_key(self, identifier)
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

    async fn wrap_key(&self, wrapping_key: PrivateKeyHandle, key: PrivateKeyHandle) -> Result<WrappedKey> {
        let pool = self.pool.clone();

        spawn::blocking(move || {
            let session = pool.get()?;
            let wrapped_key_bytes = session.wrap_key(&Mechanism::AesKeyWrapPad, wrapping_key.0, key.0)?;
            Ok(WrappedKey::new(wrapped_key_bytes))
        })
        .await
    }

    async fn unwrap_signing_key(
        &self,
        unwrapping_key: PrivateKeyHandle,
        wrapped_key: WrappedKey,
    ) -> Result<PrivateKeyHandle> {
        let pool = self.pool.clone();
        let wrapped_key: Vec<u8> = wrapped_key.into();

        spawn::blocking(move || {
            let session = pool.get()?;

            let result = session.unwrap_key(
                &Mechanism::AesKeyWrapPad,
                unwrapping_key.0,
                &wrapped_key,
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
}
