use std::sync::Arc;

use p256::ecdsa::VerifyingKey;
use p256::pkcs8::EncodePublicKey;
use tracing::debug;

use wallet_common::account::messages::auth::WalletCertificate;
use wallet_common::account::messages::auth::WalletCertificateClaims;
use wallet_common::account::serialization::DerVerifyingKey;
use wallet_common::jwt::EcdsaDecodingKey;
use wallet_common::jwt::Jwt;
use wallet_provider_domain::model::encrypted::Encrypted;
use wallet_provider_domain::model::encrypter::Decrypter;
use wallet_provider_domain::model::hsm::Hsm;
use wallet_provider_domain::model::wallet_user::WalletUser;
use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;

use crate::account_server::WalletCertificateError;
use crate::hsm::HsmError;
use crate::keys::WalletCertificateSigningKey;

const WALLET_CERTIFICATE_VERSION: u32 = 0;

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
        WalletUserQueryResult::Blocked => {
            debug!("User found for the provided certificate is blocked");
            Err(WalletCertificateError::UserBlocked)
        }
        WalletUserQueryResult::Found(user_boxed) => {
            let user = *user_boxed;
            Ok((user, claims))
        }
    }
}

pub async fn verify_wallet_certificate_public_keys<H>(
    claims: WalletCertificateClaims,
    pin_public_disclosure_protection_key_identifier: &str,
    encryption_key_identifier: &str,
    hw_pubkey: &DerVerifyingKey,
    encrypted_pin_pubkey: Encrypted<VerifyingKey>,
    hsm: &H,
) -> Result<(), WalletCertificateError>
where
    H: Decrypter<VerifyingKey, Error = HsmError> + Hsm<Error = HsmError>,
{
    debug!("Decrypt the encrypted pin public key");

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
        Err(WalletCertificateError::PinPubKeyMismatch)
    } else if *hw_pubkey != claims.hw_pubkey {
        Err(WalletCertificateError::HwPubKeyMismatch)
    } else {
        Ok(())
    }
}

pub async fn verify_wallet_certificate<T, R, H, F>(
    certificate: &WalletCertificate,
    certificate_signing_pubkey: &EcdsaDecodingKey,
    pin_public_disclosure_protection_key_identifier: &str,
    encryption_key_identifier: &str,
    wallet_user_repository: &R,
    hsm: &H,
    pin_pubkey: F,
) -> Result<WalletUser, WalletCertificateError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    H: Decrypter<VerifyingKey, Error = HsmError> + Hsm<Error = HsmError>,
    F: Fn(&WalletUser) -> Encrypted<VerifyingKey>,
{
    debug!("Parsing and verifying the provided certificate");

    let (user, claims) =
        parse_claims_and_retrieve_wallet_user(certificate, certificate_signing_pubkey, wallet_user_repository).await?;

    verify_wallet_certificate_public_keys(
        claims,
        pin_public_disclosure_protection_key_identifier,
        encryption_key_identifier,
        &user.hw_pubkey,
        pin_pubkey(&user),
        hsm,
    )
    .await?;
    Ok(user)
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

    use wallet_provider_domain::model::encrypted::Encrypted;
    use wallet_provider_domain::model::encrypter::Encrypter;
    use wallet_provider_domain::model::hsm::mock::MockPkcs11Client;
    use wallet_provider_domain::model::hsm::Hsm;

    use crate::hsm::HsmError;

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
        pub hw_privkey: SigningKey,
        pub pin_privkey: SigningKey,
        pub hw_pubkey: VerifyingKey,
        pub pin_pubkey: VerifyingKey,
        pub encrypted_pin_pubkey: Encrypted<VerifyingKey>,
        pub signing_key: SigningKey,
        pub signing_pubkey: VerifyingKey,
    }

    impl WalletCertificateSetup {
        pub async fn new() -> Self {
            let signing_key = SigningKey::random(&mut OsRng);
            let signing_pubkey = *signing_key.verifying_key();

            let hw_privkey = SigningKey::random(&mut OsRng);
            let pin_privkey = SigningKey::random(&mut OsRng);

            let hw_pubkey = *hw_privkey.verifying_key();
            let pin_pubkey = *pin_privkey.verifying_key();

            let encrypted_pin_pubkey = Encrypter::<VerifyingKey>::encrypt(
                &MockPkcs11Client::<HsmError>::default(),
                ENCRYPTION_KEY_IDENTIFIER,
                pin_pubkey,
            )
            .await
            .unwrap();

            Self {
                hw_privkey,
                pin_privkey,
                hw_pubkey,
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
    use tokio::sync::OnceCell;

    use wallet_common::jwt::EcdsaDecodingKey;
    use wallet_provider_domain::model::encrypter::Encrypter;
    use wallet_provider_domain::model::hsm::mock::MockPkcs11Client;
    use wallet_provider_persistence::repositories::mock::WalletUserTestRepo;

    use crate::hsm::HsmError;
    use crate::wallet_certificate::mock;
    use crate::wallet_certificate::new_wallet_certificate;
    use crate::wallet_certificate::sign_pin_pubkey;
    use crate::wallet_certificate::verify_pin_pubkey;
    use crate::wallet_certificate::verify_wallet_certificate;

    static HSM: OnceCell<MockPkcs11Client<HsmError>> = OnceCell::const_new();

    async fn get_global_hsm() -> &'static MockPkcs11Client<HsmError> {
        HSM.get_or_init(mock::setup_hsm).await
    }

    #[tokio::test]
    async fn sign_verify_pin_pubkey() {
        let setup = mock::WalletCertificateSetup::new().await;

        let signed = sign_pin_pubkey(
            &setup.signing_pubkey,
            mock::SIGNING_KEY_IDENTIFIER,
            get_global_hsm().await,
        )
        .await
        .unwrap();

        verify_pin_pubkey(
            &setup.signing_pubkey,
            signed,
            mock::SIGNING_KEY_IDENTIFIER,
            get_global_hsm().await,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn verify_new_wallet_certificate() {
        let setup = mock::WalletCertificateSetup::new().await;

        let wallet_certificate = new_wallet_certificate(
            String::from("issuer_1"),
            mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            &setup.signing_key,
            String::from("wallet_id_1"),
            setup.hw_pubkey,
            &setup.pin_pubkey,
            get_global_hsm().await,
        )
        .await
        .unwrap();

        verify_wallet_certificate(
            &wallet_certificate,
            &((&setup.signing_pubkey).into()),
            mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            mock::SIGNING_KEY_IDENTIFIER,
            &WalletUserTestRepo {
                hw_pubkey: setup.hw_pubkey,
                encrypted_pin_pubkey: setup.encrypted_pin_pubkey,
                previous_encrypted_pin_pubkey: None,
                challenge: None,
                instruction_sequence_number: 42,
            },
            get_global_hsm().await,
            |wallet_user| wallet_user.encrypted_pin_pubkey.clone(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn wrong_hw_key_should_not_validate() {
        let setup = mock::WalletCertificateSetup::new().await;

        let wallet_certificate = new_wallet_certificate(
            String::from("issuer_1"),
            mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            &setup.signing_key,
            String::from("wallet_id_1"),
            setup.hw_pubkey,
            &setup.pin_pubkey,
            get_global_hsm().await,
        )
        .await
        .unwrap();

        verify_wallet_certificate(
            &wallet_certificate,
            &EcdsaDecodingKey::from(&setup.signing_pubkey),
            mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            mock::ENCRYPTION_KEY_IDENTIFIER,
            &WalletUserTestRepo {
                hw_pubkey: *SigningKey::random(&mut OsRng).verifying_key(),
                encrypted_pin_pubkey: setup.encrypted_pin_pubkey,
                previous_encrypted_pin_pubkey: None,
                challenge: None,
                instruction_sequence_number: 0,
            },
            get_global_hsm().await,
            |wallet_user| wallet_user.encrypted_pin_pubkey.clone(),
        )
        .await
        .expect_err("Should not validate");
    }

    #[tokio::test]
    async fn wrong_pin_key_should_not_validate() {
        let setup = mock::WalletCertificateSetup::new().await;

        let wallet_certificate = new_wallet_certificate(
            String::from("issuer_1"),
            mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            &setup.signing_key,
            String::from("wallet_id_1"),
            setup.hw_pubkey,
            &setup.pin_pubkey,
            get_global_hsm().await,
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

        verify_wallet_certificate(
            &wallet_certificate,
            &EcdsaDecodingKey::from(&setup.signing_pubkey),
            mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            mock::ENCRYPTION_KEY_IDENTIFIER,
            &WalletUserTestRepo {
                hw_pubkey: setup.hw_pubkey,
                encrypted_pin_pubkey: other_encrypted_pin_pubkey,
                previous_encrypted_pin_pubkey: None,
                challenge: None,
                instruction_sequence_number: 0,
            },
            get_global_hsm().await,
            |wallet_user| wallet_user.encrypted_pin_pubkey.clone(),
        )
        .await
        .expect_err("Should not validate");
    }
}
