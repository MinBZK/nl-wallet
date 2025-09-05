use std::sync::Arc;

use p256::ecdsa::VerifyingKey;
use p256::pkcs8::EncodePublicKey;
use tracing::debug;

use hsm::model::Hsm;
use hsm::model::encrypted::Encrypted;
use hsm::model::encrypter::Decrypter;
use hsm::service::HsmError;
use jwt::EcdsaDecodingKey;
use jwt::Jwt;
use wallet_account::messages::registration::WalletCertificate;
use wallet_account::messages::registration::WalletCertificateClaims;
use wallet_provider_domain::model::wallet_user::WalletUser;
use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
use wallet_provider_domain::model::wallet_user::WalletUserState;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;

use crate::account_server::UserState;
use crate::account_server::WalletCertificateError;
use crate::keys::WalletCertificateSigningKey;
use crate::wua_issuer::WuaIssuer;

const WALLET_CERTIFICATE_VERSION: u32 = 0;

#[expect(clippy::too_many_arguments, reason = "Constructor of WalletCertificate")]
pub async fn new_wallet_certificate<H>(
    issuer: String,
    pin_public_disclosure_protection_key_identifier: &str,
    wallet_certificate_signing_key: &impl WalletCertificateSigningKey,
    wallet_id: String,
    wallet_hw_pubkey: VerifyingKey,
    wallet_pin_pubkey: &VerifyingKey,
    hsm: &H,
) -> Result<WalletCertificate, WalletCertificateError>
where
    H: Hsm<Error = HsmError>,
{
    let pin_pubkey_hash =
        sign_pin_pubkey(wallet_pin_pubkey, pin_public_disclosure_protection_key_identifier, hsm).await?;

    let cert = WalletCertificateClaims {
        wallet_id,
        hw_pubkey: wallet_hw_pubkey.into(),
        pin_pubkey_hash,
        version: WALLET_CERTIFICATE_VERSION,

        iss: issuer,
        iat: jsonwebtoken::get_current_timestamp(),
    };

    Jwt::sign_with_sub(&cert, wallet_certificate_signing_key)
        .await
        .map_err(WalletCertificateError::JwtSigning)
}

pub async fn parse_claims_and_retrieve_wallet_user<T, R>(
    certificate: &WalletCertificate,
    certificate_signing_pubkey: &EcdsaDecodingKey,
    wallet_user_repository: &R,
    allow_blocked: bool,
) -> Result<(WalletUser, WalletCertificateClaims), WalletCertificateError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    debug!("Parsing and verifying the provided certificate");

    let claims = certificate.parse_and_verify_with_sub(certificate_signing_pubkey)?;

    debug!("Starting database transaction");

    let tx = wallet_user_repository.begin_transaction().await?;

    debug!("Fetching the user associated to the provided certificate");

    let user_result = wallet_user_repository
        .find_wallet_user_by_wallet_id(&tx, &claims.wallet_id)
        .await?;
    tx.commit().await?;

    match user_result {
        WalletUserQueryResult::NotFound => {
            debug!("No user found for the provided certificate: {}", &claims.wallet_id);
            Err(WalletCertificateError::UserNotRegistered)
        }
        WalletUserQueryResult::Found(user_boxed)
            if !allow_blocked && matches!(user_boxed.state, WalletUserState::Blocked) =>
        {
            debug!("User found for the provided certificate is blocked");
            Err(WalletCertificateError::UserBlocked)
        }
        WalletUserQueryResult::Found(user_boxed) => {
            let user = *user_boxed;
            Ok((user, claims))
        }
    }
}

/// Specifies a PIN public key and what validations to do with it.
#[derive(Clone)]
pub enum PinKeyChecks {
    /// Verify the ECDSA signature over the instruction set with the PIN public key,
    /// and check that the HMAC of the PIN public key is present in the certificate.
    /// Normally instructions should use this variant.
    AllChecks(Encrypted<VerifyingKey>),

    /// Verify only the ECDSA signature over the instruction set with the PIN public key,
    /// and not that the HMAC of the PIN public key is present in the certificate.
    /// Only appropriate when the instruction is verified with some other PIN public key
    /// than the user's stored PIN public key.
    OnlySignature(Encrypted<VerifyingKey>),
}

impl PinKeyChecks {
    pub fn into_encrypted_verifying_key(self) -> Encrypted<VerifyingKey> {
        match self {
            PinKeyChecks::AllChecks(encrypted) => encrypted,
            PinKeyChecks::OnlySignature(encrypted) => encrypted,
        }
    }
}

pub async fn verify_wallet_certificate_public_keys<H>(
    claims: WalletCertificateClaims,
    key_identifiers: (&str, &str),
    hw_pubkey: &VerifyingKey,
    pin_checks: PinKeyChecks,
    hsm: &H,
) -> Result<(), WalletCertificateError>
where
    H: Decrypter<VerifyingKey, Error = HsmError> + Hsm<Error = HsmError>,
{
    debug!("Decrypt the encrypted pin public key");

    let (pin_public_disclosure_protection_key_identifier, encryption_key_identifier) = key_identifiers;

    if let PinKeyChecks::AllChecks(encrypted_pin_pubkey) = pin_checks {
        let pin_pubkey = Decrypter::decrypt(hsm, encryption_key_identifier, encrypted_pin_pubkey).await?;

        let pin_hash_verification = verify_pin_pubkey(
            &pin_pubkey,
            claims.pin_pubkey_hash,
            pin_public_disclosure_protection_key_identifier,
            hsm,
        )
        .await;

        debug!("Verifying the pin and hardware public keys matches those in the provided certificate");

        if pin_hash_verification.is_err() {
            return Err(WalletCertificateError::PinPubKeyMismatch);
        }
    }

    if hw_pubkey != claims.hw_pubkey.as_inner() {
        return Err(WalletCertificateError::HwPubKeyMismatch);
    }

    Ok(())
}

/// - Verify the provided [`WalletCertificate`]
/// - Retrieve the [`WalletUser`] from the DB using the `wallet_id` from the verified [`WalletCertificate`]
/// - Check that the provided PIN key and the HW key in the [`WalletUser`] are present in the
///   (verified) wallet certificate
/// - Return the [`WalletUser`].
pub async fn verify_wallet_certificate<T, R, H, F>(
    certificate: &WalletCertificate,
    certificate_signing_pubkey: &EcdsaDecodingKey,
    key_identifiers: (&str, &str),
    allow_blocked: bool,
    pin_checks: F,
    user_state: &UserState<R, H, impl WuaIssuer>,
) -> Result<(WalletUser, Encrypted<VerifyingKey>), WalletCertificateError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    H: Decrypter<VerifyingKey, Error = HsmError> + Hsm<Error = HsmError>,
    F: Fn(&WalletUser) -> PinKeyChecks,
{
    debug!("Parsing and verifying the provided certificate");

    let (user, claims) = parse_claims_and_retrieve_wallet_user(
        certificate,
        certificate_signing_pubkey,
        &user_state.repositories,
        allow_blocked,
    )
    .await?;

    let pin_checks = pin_checks(&user);

    verify_wallet_certificate_public_keys(
        claims,
        key_identifiers,
        &user.hw_pubkey,
        pin_checks.clone(),
        &user_state.wallet_user_hsm,
    )
    .await?;

    Ok((user, pin_checks.into_encrypted_verifying_key()))
}

async fn sign_pin_pubkey<H>(
    pubkey: &VerifyingKey,
    key_identifier: &str,
    hsm: &H,
) -> Result<Vec<u8>, WalletCertificateError>
where
    H: Hsm<Error = HsmError>,
{
    let pin_pubkey_bts = pubkey
        .to_public_key_der()
        .map_err(WalletCertificateError::PinPubKeyDecoding)?
        .to_vec();

    let signature = hsm.sign_hmac(key_identifier, Arc::new(pin_pubkey_bts)).await?;

    Ok(signature)
}

async fn verify_pin_pubkey<H>(
    pubkey: &VerifyingKey,
    pin_pubkey_hash: Vec<u8>,
    key_identifier: &str,
    hsm: &H,
) -> Result<(), WalletCertificateError>
where
    H: Hsm<Error = HsmError>,
{
    let pin_pubkey_bts = pubkey
        .to_public_key_der()
        .map_err(WalletCertificateError::PinPubKeyDecoding)?
        .to_vec();

    hsm.verify_hmac(key_identifier, Arc::new(pin_pubkey_bts), pin_pubkey_hash)
        .await?;

    Ok(())
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use hmac::digest::crypto_common::rand_core::OsRng;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;

    use hsm::model::Hsm;
    use hsm::model::encrypted::Encrypted;
    use hsm::model::encrypter::Encrypter;
    use hsm::model::mock::MockPkcs11Client;
    use hsm::service::HsmError;

    pub const SIGNING_KEY_IDENTIFIER: &str = "certificate_signing_key_1";
    pub const PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER: &str =
        "pin_public_disclosure_protection_key_identifier_1";
    pub const ENCRYPTION_KEY_IDENTIFIER: &str = "encryption_key_1";

    pub async fn setup_hsm() -> MockPkcs11Client<HsmError> {
        let hsm = MockPkcs11Client::default();
        hsm.generate_generic_secret_key(SIGNING_KEY_IDENTIFIER).await.unwrap();
        hsm.generate_generic_secret_key(PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER)
            .await
            .unwrap();
        hsm
    }

    #[derive(Clone)]
    pub struct WalletCertificateSetup {
        pub pin_privkey: SigningKey,
        pub pin_pubkey: VerifyingKey,
        pub encrypted_pin_pubkey: Encrypted<VerifyingKey>,
        pub signing_key: SigningKey,
        pub signing_pubkey: VerifyingKey,
    }

    impl WalletCertificateSetup {
        pub async fn new() -> Self {
            let pin_privkey = SigningKey::random(&mut OsRng);
            let pin_pubkey = *pin_privkey.verifying_key();

            let signing_key = SigningKey::random(&mut OsRng);
            let signing_pubkey = *signing_key.verifying_key();

            let encrypted_pin_pubkey = Encrypter::<VerifyingKey>::encrypt(
                &MockPkcs11Client::<HsmError>::default(),
                ENCRYPTION_KEY_IDENTIFIER,
                pin_pubkey,
            )
            .await
            .unwrap();

            Self {
                pin_privkey,
                pin_pubkey,
                encrypted_pin_pubkey,
                signing_key,
                signing_pubkey,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use hmac::digest::crypto_common::rand_core::OsRng;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;

    use hsm::model::encrypter::Encrypter;
    use hsm::model::mock::MockPkcs11Client;
    use hsm::service::HsmError;
    use jwt::EcdsaDecodingKey;
    use wallet_provider_domain::model::wallet_user::WalletUserState;
    use wallet_provider_persistence::repositories::mock::WalletUserTestRepo;

    use crate::account_server::mock::user_state;
    use crate::wallet_certificate::PinKeyChecks;
    use crate::wallet_certificate::mock;
    use crate::wallet_certificate::mock::setup_hsm;
    use crate::wallet_certificate::new_wallet_certificate;
    use crate::wallet_certificate::sign_pin_pubkey;
    use crate::wallet_certificate::verify_pin_pubkey;
    use crate::wallet_certificate::verify_wallet_certificate;

    #[tokio::test]
    async fn sign_verify_pin_pubkey() {
        let setup = mock::WalletCertificateSetup::new().await;
        let hsm = setup_hsm().await;

        let signed = sign_pin_pubkey(&setup.signing_pubkey, mock::SIGNING_KEY_IDENTIFIER, &hsm)
            .await
            .unwrap();

        verify_pin_pubkey(&setup.signing_pubkey, signed, mock::SIGNING_KEY_IDENTIFIER, &hsm)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn verify_new_wallet_certificate() {
        let setup = mock::WalletCertificateSetup::new().await;
        let hsm = setup_hsm().await;
        let hw_pubkey = *SigningKey::random(&mut OsRng).verifying_key();
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let wallet_certificate = new_wallet_certificate(
            String::from("issuer_1"),
            mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            &setup.signing_key,
            String::from("wallet_id_1"),
            hw_pubkey,
            &setup.pin_pubkey,
            &hsm,
        )
        .await
        .unwrap();

        let user_state = user_state(
            WalletUserTestRepo {
                hw_pubkey,
                encrypted_pin_pubkey: setup.encrypted_pin_pubkey,
                previous_encrypted_pin_pubkey: None,
                challenge: None,
                instruction_sequence_number: 42,
                apple_assertion_counter: None,
                state: WalletUserState::Active,
            },
            hsm,
            wrapping_key_identifier.to_string(),
        );

        verify_wallet_certificate(
            &wallet_certificate,
            &((&setup.signing_pubkey).into()),
            (
                mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
                mock::SIGNING_KEY_IDENTIFIER,
            ),
            false,
            |wallet_user| PinKeyChecks::AllChecks(wallet_user.encrypted_pin_pubkey.clone()),
            &user_state,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn wrong_hw_key_should_not_validate() {
        let setup = mock::WalletCertificateSetup::new().await;
        let hsm = setup_hsm().await;
        let hw_pubkey = *SigningKey::random(&mut OsRng).verifying_key();
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let wallet_certificate = new_wallet_certificate(
            String::from("issuer_1"),
            mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            &setup.signing_key,
            String::from("wallet_id_1"),
            hw_pubkey,
            &setup.pin_pubkey,
            &hsm,
        )
        .await
        .unwrap();

        let user_state = user_state(
            WalletUserTestRepo {
                hw_pubkey: *SigningKey::random(&mut OsRng).verifying_key(),
                encrypted_pin_pubkey: setup.encrypted_pin_pubkey,
                previous_encrypted_pin_pubkey: None,
                challenge: None,
                instruction_sequence_number: 0,
                apple_assertion_counter: None,
                state: WalletUserState::Active,
            },
            setup_hsm().await,
            wrapping_key_identifier.to_string(),
        );

        verify_wallet_certificate(
            &wallet_certificate,
            &EcdsaDecodingKey::from(&setup.signing_pubkey),
            (
                mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
                mock::ENCRYPTION_KEY_IDENTIFIER,
            ),
            false,
            |wallet_user| PinKeyChecks::AllChecks(wallet_user.encrypted_pin_pubkey.clone()),
            &user_state,
        )
        .await
        .expect_err("certificate with incorrect hardware key should not validate");
    }

    #[tokio::test]
    async fn wrong_pin_key_should_not_validate() {
        let setup = mock::WalletCertificateSetup::new().await;
        let hsm = setup_hsm().await;
        let hw_pubkey = *SigningKey::random(&mut OsRng).verifying_key();
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let wallet_certificate = new_wallet_certificate(
            String::from("issuer_1"),
            mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            &setup.signing_key,
            String::from("wallet_id_1"),
            hw_pubkey,
            &setup.pin_pubkey,
            &hsm,
        )
        .await
        .unwrap();

        let other_encrypted_pin_pubkey = Encrypter::<VerifyingKey>::encrypt(
            &MockPkcs11Client::<HsmError>::default(),
            mock::ENCRYPTION_KEY_IDENTIFIER,
            *SigningKey::random(&mut OsRng).verifying_key(),
        )
        .await
        .unwrap();

        let user_state = user_state(
            WalletUserTestRepo {
                hw_pubkey,
                encrypted_pin_pubkey: other_encrypted_pin_pubkey,
                previous_encrypted_pin_pubkey: None,
                challenge: None,
                instruction_sequence_number: 0,
                apple_assertion_counter: None,
                state: WalletUserState::Active,
            },
            hsm,
            wrapping_key_identifier.to_string(),
        );

        verify_wallet_certificate(
            &wallet_certificate,
            &EcdsaDecodingKey::from(&setup.signing_pubkey),
            (
                mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
                mock::ENCRYPTION_KEY_IDENTIFIER,
            ),
            false,
            |wallet_user| PinKeyChecks::AllChecks(wallet_user.encrypted_pin_pubkey.clone()),
            &user_state,
        )
        .await
        .expect_err("certificate with incorrect PIN key HMAC should not validate");
    }
}
