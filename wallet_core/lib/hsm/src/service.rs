use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::Duration;

use crypto::aes_siv::AesSivBackend;
use crypto::p256_der::verifying_key_sha256;
use crypto::utils::random_bytes;
use crypto::utils::sha256;
use cryptoki::context::CInitializeArgs;
use cryptoki::context::CInitializeFlags;
use cryptoki::context::Pkcs11;
use cryptoki::mechanism::Mechanism;
use cryptoki::mechanism::MechanismType;
use cryptoki::mechanism::aead::GcmParams;
use cryptoki::mechanism::vendor_defined::VendorDefinedMechanism;
use cryptoki::object::Attribute;
use cryptoki::object::AttributeType;
use cryptoki::object::KeyType;
use cryptoki::object::ObjectClass;
use cryptoki::object::ObjectHandle;
use cryptoki::types::AuthPin;
use cryptoki_sys::CK_AES_CTR_PARAMS;
use derive_more::AsRef;
use futures::future;
use measure::measure;
use p256::NistP256;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::AssociatedOid;
use r2d2_cryptoki::Pool;
use r2d2_cryptoki::SessionAuth;
use r2d2_cryptoki::SessionManager;
use r2d2_cryptoki::r2d2::LoggingErrorHandler;
use sec1::EcParameters;
use sec1::der::Decode;
use sec1::der::Encode;
use sec1::der::asn1::OctetStringRef;
use utils::spawn;
use utils::vec_at_least::VecNonEmpty;

use crate::model::Hsm;
use crate::model::encrypted::Encrypted;
use crate::model::encrypted::InitializationVector;
use crate::model::wrapped_key::WrappedKey;
use crate::settings;

#[derive(Debug, thiserror::Error, strum::IntoStaticStr)]
pub enum HsmError {
    #[error("pkcs11 error: {0}")]
    Pkcs11(#[from] cryptoki::error::Error),

    #[error("r2d2 error: {0}")]
    R2d2(#[from] r2d2_cryptoki::r2d2::Error),

    #[error("sec1 error: {0}")]
    Sec1(#[source] Box<sec1::der::Error>),

    #[error("no initialized slot available")]
    NoInitializedSlotAvailable,

    #[error("p256 error: {0}")]
    P256(#[from] p256::ecdsa::Error),

    #[error("attribute not found: '{0}'")]
    AttributeNotFound(String),

    #[error("key not found: '{0}'")]
    KeyNotFound(String),

    #[error("CMAC has wrong length: expected {expected}, got {actual}")]
    IncorrectCmacLength { expected: usize, actual: usize },

    #[cfg(feature = "mock")]
    #[error("hmac error: {0}")]
    Hmac(#[from] hmac::digest::MacError),
}

#[cfg(feature = "test")]
impl HsmError {
    /// Returns `true` if this error, as returned by [`Pkcs11Hsm::import_aes_key()`], means that the
    /// token refuses to accept externally supplied key material at all. It returns `false` for all
    /// other errors, e.g. when the token does support key import but this particular call was
    /// malformed.
    ///
    /// Importing a key is only ever done to run known-answer tests, and production HSMs commonly
    /// forbid it by policy, so `true` means "skip these tests" rather than "something went wrong".
    pub fn is_key_import_unsupported(&self) -> bool {
        use cryptoki::error::Error;
        use cryptoki::error::RvError;

        matches!(
            self,
            HsmError::Pkcs11(Error::Pkcs11(
                RvError::ActionProhibited
                    | RvError::AttributeReadOnly
                    | RvError::FunctionNotSupported
                    | RvError::TemplateInconsistent,
                _,
            ))
        )
    }
}

type Result<T> = std::result::Result<T, HsmError>;

/// PrivateKeyHandle that wraps ObjectHandle for private keys
///
/// Note that this struct doesn't derive Copy (ObjectHandle does) on purpose to
/// leverage the type system to detect when handles are destroyed.
#[derive(PartialEq, Eq)]
pub struct PrivateKeyHandle(ObjectHandle);

/// PublicKeyHandle that wraps ObjectHandle for public keys
///
/// Note that this struct doesn't derive Copy (ObjectHandle does) on purpose to
/// leverage the type system to detect when handles are destroyed.
pub struct PublicKeyHandle(ObjectHandle);

pub trait KeyHandle: Send + 'static {
    fn to_object_handle(&self) -> ObjectHandle;
}

impl KeyHandle for PrivateKeyHandle {
    fn to_object_handle(&self) -> ObjectHandle {
        self.0
    }
}

impl KeyHandle for PublicKeyHandle {
    fn to_object_handle(&self) -> ObjectHandle {
        self.0
    }
}

const AES_AUTHENTICATION_TAG_BITS: u64 = 128;
pub const AES_BLOCK_SIZE: usize = 16;
const AES_CTR_COUNTER_BITS: u64 = (AES_BLOCK_SIZE * 8) as u64;

enum HandleType {
    Public,
    Private,
}

pub enum SigningMechanism {
    Ecdsa256,
    Sha256Hmac,
}

#[derive(Debug, Clone, Copy)]
pub enum AesKeyUsage {
    Encrypt,
    Cmac,
}

impl AesKeyUsage {
    fn attribute(self) -> Attribute {
        match self {
            AesKeyUsage::Encrypt => Attribute::Encrypt(true),
            AesKeyUsage::Cmac => Attribute::Sign(true),
        }
    }
}

pub trait Pkcs11Client {
    async fn generate_aes_key(&self, identifier: &str, usage: AesKeyUsage) -> Result<PrivateKeyHandle>;
    async fn generate_generic_secret_key(&self, identifier: &str) -> Result<PrivateKeyHandle>;
    async fn generate_session_signing_key_pair(&self) -> Result<(PublicKeyHandle, PrivateKeyHandle)>;
    async fn generate_signing_key_pair(&self, identifier: &str) -> Result<(PublicKeyHandle, PrivateKeyHandle)>;
    async fn get_private_key_handle(&self, identifier: &str) -> Result<PrivateKeyHandle>;
    async fn get_public_key_handle(&self, identifier: &str) -> Result<PublicKeyHandle>;
    async fn get_verifying_key(&self, public_key_handle: &PublicKeyHandle) -> Result<VerifyingKey>;
    /// Delete key (takes ownership since the handle is invalid after deletion)
    async fn delete_key(&self, key_handle: impl KeyHandle) -> Result<()>;
    async fn sign(
        &self,
        private_key_handle: &PrivateKeyHandle,
        mechanism: SigningMechanism,
        data: &[u8],
    ) -> Result<Vec<u8>>;
    async fn verify(
        &self,
        private_key_handle: &PrivateKeyHandle,
        mechanism: SigningMechanism,
        data: &[u8],
        signature: Vec<u8>,
    ) -> Result<()>;
    async fn encrypt(
        &self,
        key_handle: &PrivateKeyHandle,
        iv: InitializationVector,
        data: Vec<u8>,
    ) -> Result<(Vec<u8>, InitializationVector)>;
    async fn decrypt(
        &self,
        key_handle: &PrivateKeyHandle,
        iv: InitializationVector,
        encrypted_data: Vec<u8>,
    ) -> Result<Vec<u8>>;
    /// AES-CTR-256, starting from `counter_block`. Note that AES-CTR is symmetric, so this one function serves
    /// for both encryption and decryption.
    // The counter block is the caller's to choose, and implementations must use it exactly as given: one that
    // generates its own, or otherwise doesn't use `counter_block` exactly as given, is not interchangeable with
    // the others, and could lead to silent interoperability failures with other implementations.
    async fn encrypt_ctr(
        &self,
        key_handle: &PrivateKeyHandle,
        counter_block: [u8; AES_BLOCK_SIZE],
        data: impl AsRef<[u8]> + Send + 'static,
    ) -> Result<Vec<u8>>;
    async fn cmac(
        &self,
        key_handle: &PrivateKeyHandle,
        data: impl AsRef<[u8]> + Send + 'static,
    ) -> Result<[u8; AES_BLOCK_SIZE]>;
    async fn wrap_key(
        &self,
        wrapping_key: &PrivateKeyHandle,
        key: &PrivateKeyHandle,
        public_key: VerifyingKey,
    ) -> Result<WrappedKey>;
    async fn unwrap_signing_key(
        &self,
        unwrapping_key: &PrivateKeyHandle,
        wrapped_key: WrappedKey,
    ) -> Result<PrivateKeyHandle>;
    async fn generate_wrapped_key(&self, wrapping_key_identifier: &str) -> Result<WrappedKey>;
    async fn generate_wrapped_keys(
        &self,
        wrapping_key_identifier: &str,
        count: NonZeroUsize,
    ) -> Result<VecNonEmpty<(String, WrappedKey)>> {
        future::try_join_all((0..count.get()).map(|_| async move {
            let result = self.generate_wrapped_key(wrapping_key_identifier).await;
            result.map(|wrapped| (verifying_key_sha256(wrapped.public_key()), wrapped))
        }))
        .await
        .map(|keys| {
            // Unwrap is safe because we generated `count` keys, which is a nonzero type
            keys.try_into().unwrap()
        })
    }
    async fn sign_wrapped(
        &self,
        wrapping_key_identifier: &str,
        wrapped_key: WrappedKey,
        data: &[u8],
    ) -> Result<Signature>;
}

#[derive(Clone, AsRef)]
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
        pkcs11_client.initialize(CInitializeArgs::new(CInitializeFlags::OS_LOCKING_OK))?;

        let slot = *pkcs11_client
            .get_slots_with_initialized_token()?
            .first()
            .ok_or(HsmError::NoInitializedSlotAvailable)?;

        let session_auth = SessionAuth::RwUser(AuthPin::from(user_pin));
        let manager = SessionManager::new(pkcs11_client, slot, &session_auth);

        let pool = Pool::builder()
            .max_size(max_sessions.into())
            .max_lifetime(Some(max_session_lifetime))
            // This makes a pkcs11 call every time a connection is check out of the pool and should be evaluated in a
            // future performance test.
            .test_on_check_out(true)
            .connection_customizer(session_auth.into_customizer())
            .error_handler(Box::new(LoggingErrorHandler))
            .build(manager)?;

        Ok(Self { pool })
    }

    pub fn from_settings(settings: settings::Hsm) -> Result<Self> {
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

    async fn generate_aes_key(&self, identifier: &str, usage: AesKeyUsage) -> std::result::Result<(), Self::Error> {
        Pkcs11Client::generate_aes_key(self, identifier, usage)
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
        Pkcs11Client::get_verifying_key(self, &handle).await
    }

    async fn delete_key(&self, identifier: &str) -> Result<()> {
        let handle = self.get_private_key_handle(identifier).await?;
        Pkcs11Client::delete_key(self, handle).await?;
        Ok(())
    }

    async fn sign_ecdsa(&self, identifier: &str, data: &[u8]) -> std::result::Result<Signature, Self::Error> {
        let handle = self.get_private_key_handle(identifier).await?;
        let signature = Pkcs11Client::sign(self, &handle, SigningMechanism::Ecdsa256, data).await?;
        Ok(Signature::from_slice(&signature)?)
    }

    async fn sign_hmac(&self, identifier: &str, data: &[u8]) -> std::result::Result<Vec<u8>, Self::Error> {
        let handle = self.get_private_key_handle(identifier).await?;
        Pkcs11Client::sign(self, &handle, SigningMechanism::Sha256Hmac, data).await
    }

    async fn verify_hmac(
        &self,
        identifier: &str,
        data: &[u8],
        signature: Vec<u8>,
    ) -> std::result::Result<(), Self::Error> {
        let handle = self.get_private_key_handle(identifier).await?;
        Pkcs11Client::verify(self, &handle, SigningMechanism::Sha256Hmac, data, signature).await
    }

    async fn encrypt<T>(&self, identifier: &str, data: Vec<u8>) -> Result<Encrypted<T>> {
        let iv = random_bytes(32);
        let handle = self.get_private_key_handle(identifier).await?;
        let (encrypted_data, initialization_vector) =
            Pkcs11Client::encrypt(self, &handle, InitializationVector(iv), data).await?;
        Ok(Encrypted::new(encrypted_data, initialization_vector))
    }

    async fn decrypt<T>(&self, identifier: &str, encrypted: Encrypted<T>) -> Result<Vec<u8>> {
        let handle = self.get_private_key_handle(identifier).await?;
        Pkcs11Client::decrypt(self, &handle, encrypted.iv, encrypted.data).await
    }
}

impl Pkcs11Client for Pkcs11Hsm {
    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
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

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn generate_aes_key(&self, identifier: &str, usage: AesKeyUsage) -> Result<PrivateKeyHandle> {
        let pool = self.pool.clone();
        let identifier = String::from(identifier);

        spawn::blocking(move || {
            let session = pool.get()?;

            let priv_key_template = &[
                usage.attribute(),
                Attribute::Token(true),
                Attribute::Private(true),
                Attribute::Sensitive(true),
                Attribute::Extractable(false),
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

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn generate_session_signing_key_pair(&self) -> Result<(PublicKeyHandle, PrivateKeyHandle)> {
        let pool = self.pool.clone();

        spawn::blocking(move || {
            let session = pool.get()?;

            let mut ec_params = vec![];
            EcParameters::NamedCurve(NistP256::OID)
                .encode_to_vec(&mut ec_params)
                .map_err(|error| HsmError::Sec1(Box::new(error)))?;

            let pub_key_template = &[
                Attribute::EcParams(ec_params),
                Attribute::Token(false),
                Attribute::Private(false),
            ];
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

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn generate_signing_key_pair(&self, identifier: &str) -> Result<(PublicKeyHandle, PrivateKeyHandle)> {
        let pool = self.pool.clone();
        let identifier = String::from(identifier);

        spawn::blocking(move || {
            let session = pool.get()?;

            let mut ec_params = vec![];
            EcParameters::NamedCurve(NistP256::OID)
                .encode_to_vec(&mut ec_params)
                .map_err(|error| HsmError::Sec1(Box::new(error)))?;

            let pub_key_template = &[
                Attribute::EcParams(ec_params),
                Attribute::Token(true),
                Attribute::Private(false),
                Attribute::Label(identifier.clone().into()),
            ];
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

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn get_private_key_handle(&self, identifier: &str) -> Result<PrivateKeyHandle> {
        self.get_key_handle(identifier, HandleType::Private)
            .await
            .map(PrivateKeyHandle)
    }

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn get_public_key_handle(&self, identifier: &str) -> Result<PublicKeyHandle> {
        self.get_key_handle(identifier, HandleType::Public)
            .await
            .map(PublicKeyHandle)
    }

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn get_verifying_key(&self, public_key_handle: &PublicKeyHandle) -> Result<VerifyingKey> {
        let pool = self.pool.clone();
        let object_handle = public_key_handle.to_object_handle();

        spawn::blocking(move || {
            let session = pool.get()?;
            let attr = session
                .get_attributes(object_handle, &[AttributeType::EcPoint])?
                .first()
                .cloned()
                .ok_or(HsmError::AttributeNotFound(AttributeType::EcPoint.to_string()))?;

            match attr {
                Attribute::EcPoint(ec_point) => {
                    let octet_string =
                        <&OctetStringRef>::from_der(&ec_point).map_err(|error| HsmError::Sec1(Box::new(error)))?;
                    let public_key = VerifyingKey::from_sec1_bytes(octet_string.as_bytes())?;
                    Ok(public_key)
                }
                _ => Err(HsmError::AttributeNotFound(AttributeType::EcPoint.to_string())),
            }
        })
        .await
    }

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn delete_key(&self, key_handle: impl KeyHandle) -> Result<()> {
        let pool = self.pool.clone();

        spawn::blocking(move || {
            let session = pool.get()?;
            session.destroy_object(key_handle.to_object_handle())?;
            Ok(())
        })
        .await
    }

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn sign(
        &self,
        private_key_handle: &PrivateKeyHandle,
        mechanism: SigningMechanism,
        data: &[u8],
    ) -> Result<Vec<u8>> {
        let pool = self.pool.clone();
        let data_hash = sha256(data);
        let object_handle = private_key_handle.to_object_handle();

        spawn::blocking(move || {
            let mechanism = match mechanism {
                SigningMechanism::Ecdsa256 => Mechanism::Ecdsa,
                SigningMechanism::Sha256Hmac => Mechanism::Sha256Hmac,
            };

            let session = pool.get()?;
            let signature = session.sign(&mechanism, object_handle, &data_hash)?;
            Ok(signature)
        })
        .await
    }

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn verify(
        &self,
        private_key_handle: &PrivateKeyHandle,
        mechanism: SigningMechanism,
        data: &[u8],
        signature: Vec<u8>,
    ) -> Result<()> {
        let pool = self.pool.clone();
        let data_hash = sha256(data);
        let object_handle = private_key_handle.to_object_handle();

        spawn::blocking(move || {
            let mechanism = match mechanism {
                SigningMechanism::Ecdsa256 => Mechanism::Ecdsa,
                SigningMechanism::Sha256Hmac => Mechanism::Sha256Hmac,
            };

            let session = pool.get()?;
            session.verify(&mechanism, object_handle, &data_hash, &signature)?;

            Ok(())
        })
        .await
    }

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn encrypt(
        &self,
        key_handle: &PrivateKeyHandle,
        mut iv: InitializationVector,
        data: Vec<u8>,
    ) -> Result<(Vec<u8>, InitializationVector)> {
        let pool = self.pool.clone();
        let object_handle = key_handle.to_object_handle();

        spawn::blocking(move || {
            let session = pool.get()?;
            let gcm_params = GcmParams::new(iv.0.as_mut_slice(), &[], AES_AUTHENTICATION_TAG_BITS.into())?;
            let encrypted_data = session.encrypt(&Mechanism::AesGcm(gcm_params), object_handle, &data)?;
            Ok((encrypted_data, iv))
        })
        .await
    }

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn decrypt(
        &self,
        key_handle: &PrivateKeyHandle,
        mut iv: InitializationVector,
        encrypted_data: Vec<u8>,
    ) -> Result<Vec<u8>> {
        let pool = self.pool.clone();
        let object_handle = key_handle.to_object_handle();

        spawn::blocking(move || {
            let session = pool.get()?;
            let gcm_params = GcmParams::new(iv.0.as_mut_slice(), &[], AES_AUTHENTICATION_TAG_BITS.into())?;
            let data = session.decrypt(&Mechanism::AesGcm(gcm_params), object_handle, &encrypted_data)?;
            Ok(data)
        })
        .await
    }

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn encrypt_ctr(
        &self,
        key_handle: &PrivateKeyHandle,
        counter_block: [u8; AES_BLOCK_SIZE],
        data: impl AsRef<[u8]> + Send + 'static,
    ) -> Result<Vec<u8>> {
        let pool = self.pool.clone();
        let object_handle = key_handle.to_object_handle();

        spawn::blocking(move || {
            let session = pool.get()?;

            let params = CK_AES_CTR_PARAMS {
                ulCounterBits: AES_CTR_COUNTER_BITS,
                cb: counter_block, // the initial counter block
            };

            // `cryptoki` has no `Mechanism` variant for AES-CTR, so the parameters are passed
            // through `VendorDefinedMechanism`. Despite its name it pairs any mechanism type with
            // any parameter struct, by passing a raw pointer to `params`.
            //
            // This is essentially an untyped escape hatch, passing a struct from the `cryptoki_sys` package to
            // a `cryptoki` API.
            let mechanism =
                Mechanism::VendorDefined(VendorDefinedMechanism::new(MechanismType::AES_CTR, Some(&params)));

            let encrypted_data = session.encrypt(&mechanism, object_handle, data.as_ref())?;
            Ok(encrypted_data)
        })
        .await
    }

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn cmac(
        &self,
        key_handle: &PrivateKeyHandle,
        data: impl AsRef<[u8]> + Send + 'static,
    ) -> Result<[u8; AES_BLOCK_SIZE]> {
        let pool = self.pool.clone();
        let object_handle = key_handle.to_object_handle();

        spawn::blocking(move || {
            let session = pool.get()?;
            let cmac = session.sign(&Mechanism::AesCMac, object_handle, data.as_ref())?;

            let cmac = cmac.try_into().map_err(|cmac: Vec<u8>| HsmError::IncorrectCmacLength {
                expected: AES_BLOCK_SIZE,
                actual: cmac.len(),
            })?;

            Ok(cmac)
        })
        .await
    }

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn wrap_key(
        &self,
        wrapping_key: &PrivateKeyHandle,
        key: &PrivateKeyHandle,
        public_key: VerifyingKey,
    ) -> Result<WrappedKey> {
        let pool = self.pool.clone();
        let wrapping_key_handle = wrapping_key.to_object_handle();
        let key_handle = key.to_object_handle();

        spawn::blocking(move || {
            let session = pool.get()?;
            let wrapped_key_bytes = session.wrap_key(&Mechanism::AesKeyWrapPad, wrapping_key_handle, key_handle)?;
            Ok(WrappedKey::new(wrapped_key_bytes, public_key))
        })
        .await
    }

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn unwrap_signing_key(
        &self,
        unwrapping_key: &PrivateKeyHandle,
        wrapped_key: WrappedKey,
    ) -> Result<PrivateKeyHandle> {
        let pool = self.pool.clone();
        let unwrapping_key_handle = unwrapping_key.to_object_handle();

        spawn::blocking(move || {
            let session = pool.get()?;

            let result = session.unwrap_key(
                &Mechanism::AesKeyWrapPad,
                unwrapping_key_handle,
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

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn generate_wrapped_key(&self, wrapping_key_identifier: &str) -> Result<WrappedKey> {
        // TODO: PVW-5862 handles can be used in different sessions that are
        // either connected to a different PKCS11 device or the session can be
        // closed by the pool which will cause the deletion of session objects.
        let private_wrapping_handle = self.get_private_key_handle(wrapping_key_identifier).await?;
        let (public_handle, private_handle) = self.generate_session_signing_key_pair().await?;
        let verifying_key = Pkcs11Client::get_verifying_key(self, &public_handle).await?;

        let result = self
            .wrap_key(&private_wrapping_handle, &private_handle, verifying_key)
            .await;

        self.clone().delete_keypair_in_background(private_handle, public_handle);

        result
    }

    #[measure(name = "nlwallet_pkcs11_operations", "service" => "pkcs11")]
    async fn sign_wrapped(
        &self,
        wrapping_key_identifier: &str,
        wrapped_key: WrappedKey,
        data: &[u8],
    ) -> Result<Signature> {
        // TODO: PVW-5862 handles can be used in different sessions that are
        // either connected to a different PKCS11 device or the session can be
        // closed by the pool which will cause the deletion of session objects.

        let private_wrapping_handle = self.get_private_key_handle(wrapping_key_identifier).await?;
        let private_handle = self.unwrap_signing_key(&private_wrapping_handle, wrapped_key).await?;
        let result = Pkcs11Client::sign(self, &private_handle, SigningMechanism::Ecdsa256, data).await;
        self.clone().delete_private_key_in_background(private_handle);
        result.and_then(|signature| Signature::from_slice(&signature).map_err(HsmError::from))
    }
}

impl AesSivBackend for Pkcs11Hsm {
    type Error = HsmError;

    type MacKey = PrivateKeyHandle;
    type EncryptionKey = PrivateKeyHandle;

    async fn aes_cmac(&self, key: &Self::MacKey, input: impl AsRef<[u8]> + Send + 'static) -> Result<[u8; 16]> {
        self.cmac(key, input).await
    }

    async fn aes_ctr(
        &self,
        key: &Self::EncryptionKey,
        counter_block: [u8; 16],
        input: impl AsRef<[u8]> + Send + 'static,
    ) -> Result<Vec<u8>> {
        self.encrypt_ctr(key, counter_block, input).await
    }
}

#[cfg(feature = "test")]
impl Pkcs11Hsm {
    /// Imports an AES-256 key with a caller-chosen value, so that known-answer tests have a key
    /// whose value is known on both sides. There is no production use for this: every other key in
    /// this module is generated by the token and never leaves it.
    ///
    /// Not all tokens allow this, and those that refuse do so by policy rather than because the
    /// call was wrong; see [`HsmError::is_key_import_unsupported()`].
    pub async fn import_aes_key(
        &self,
        identifier: &str,
        usage: AesKeyUsage,
        key: [u8; 32],
    ) -> Result<PrivateKeyHandle> {
        let pool = self.pool.clone();
        let identifier = String::from(identifier);

        spawn::blocking(move || {
            let session = pool.get()?;

            // The same template as `generate_aes_key()`, except that the key's value is supplied
            // rather than only its length.
            let template = &[
                usage.attribute(),
                Attribute::Token(true),
                Attribute::Private(true),
                Attribute::Sensitive(true),
                Attribute::Extractable(false),
                Attribute::Class(ObjectClass::SECRET_KEY),
                Attribute::KeyType(KeyType::AES),
                Attribute::Value(key.to_vec()),
                Attribute::Label(identifier.clone().into()),
            ];

            let handle = session.create_object(template)?;

            Ok(PrivateKeyHandle(handle))
        })
        .await
    }
}

impl Pkcs11Hsm {
    fn delete_private_key_in_background(self, private_handle: PrivateKeyHandle) {
        tokio::spawn(async move {
            if let Err(err) = Pkcs11Client::delete_key(&self, private_handle).await {
                tracing::warn!("failed to delete private key: {err:?}");
            }
        });
    }

    fn delete_keypair_in_background(self, private_key_handle: PrivateKeyHandle, public_key_handle: PublicKeyHandle) {
        tokio::spawn(async move {
            if let Err(err) = Pkcs11Client::delete_key(&self, private_key_handle).await {
                tracing::warn!("failed to delete private key: {err:?}");
            }
            if let Err(err) = Pkcs11Client::delete_key(&self, public_key_handle).await {
                tracing::warn!("failed to delete public key: {err:?}");
            }
        });
    }
}
