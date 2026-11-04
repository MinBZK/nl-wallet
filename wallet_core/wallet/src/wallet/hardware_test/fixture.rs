use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use apple_app_attest::AppIdentifier;
use apple_app_attest::AttestationEnvironment;
use attestation_data::attributes::Attribute;
use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::registration_certificate::ParsedRegistrationCertificate;
use attestation_data::registration_certificate::mock::MockRegistrationCertificateAuthority;
use attestation_data::registration_certificate::mock::STATUS_LIST_URI;
use attestation_data::registration_certificate::mock::verifier_registration_certificate_payload;
use attestation_data::validity::ValidityWindow;
use attestation_types::pid_constants::PID_AGE_OVER_18;
use attestation_types::pid_constants::PID_BIRTH_DATE;
use attestation_types::pid_constants::PID_BSN;
use attestation_types::pid_constants::PID_FAMILY_NAME;
use attestation_types::pid_constants::PID_GIVEN_NAME;
use attestation_types::pid_constants::PID_MDOC_DOCTYPE;
use attestation_types::pid_constants::PID_RECOVERY_CODE;
use attestation_types::status_claim::StatusListClaim;
use chrono::TimeDelta;
use chrono::Utc;
use ciborium::value::Value;
use crypto::keys::PublicKeyRef;
use crypto::p256_der::DerSignature;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::x509::crl::mock::MockCertificateCrlVerifier;
use dcql::Query;
use futures::FutureExt;
use jwt::KeyWithKid;
use jwt::SignedJwt;
use mdoc::holder::Mdoc;
use mockall::predicate::eq;
use openid4vc::disclosure_session::mock::MockDisclosureClient;
use openid4vc::metadata::issuer_metadata::CredentialMetadata;
use openid4vc::wallet_issuance::credential::CredentialWithMetadata;
use openid4vc::wallet_issuance::credential::IssuedCredentialCopies;
use openid4vc::wallet_issuance::credential::IssuedCredentialMetadata;
use openid4vc::wallet_issuance::credential::MdocCopy;
use openid4vc::wallet_issuance::issuer_registration::IssuerRegistration;
use openid4vc::wallet_issuance::mock::MockIssuanceDiscovery;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::signature::hazmat::PrehashSigner;
use p256::elliptic_curve::Generate;
use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::mock::MockHardwareAttestedKeyHolder;
use token_status_list::status_list::StatusList;
use token_status_list::status_list_jwt::StatusListJwt;
use token_status_list::status_list_token::StatusListToken;
use token_status_list::verification::client::mock::MockStatusListClient;
use utils::generator::TimeGenerator;
use utils::vec_nonempty;
use wallet_account::RevocationCode;
use wallet_account::messages::instructions::Instruction;
use wallet_account::messages::instructions::InstructionResultClaims;
use wallet_account::messages::instructions::Sign;
use wallet_account::messages::instructions::SignResult;
use wallet_account::messages::registration::WalletCertificateClaims;
use wallet_configuration::wallet_config::CertificatePublicKey;
use wallet_configuration::wallet_config::WalletConfiguration;

use super::CREDENTIAL_STATUS_LIST_URI;
use super::EXPECTED_GIVEN_NAME;
use super::RegistrationCertificateCase;
use super::Scenario;
use super::Verification;
use crate::AttestationIdentity;
use crate::AttestationPresentation;
use crate::account_provider::MockAccountProviderClient;
use crate::attestation::AttestationValidity;
use crate::config::LocalConfigurationRepository;
use crate::config::UpdatingConfigurationRepository;
use crate::config::default_config_server_config;
use crate::pin::key::new_pin_salt;
use crate::storage::MockHardwareDatabaseStorage;
use crate::storage::RegistrationData;
use crate::storage::Storage;
use crate::update_policy::MockUpdatePolicyRepository;
use crate::wallet::Wallet;
use crate::wallet::WalletRegistration;
use crate::wallet::init::RegistrationStatus;
use crate::wallet::init::WalletClients;
use crate::wallet::init::WalletRepositories;

const HOLDER_KEY_ID: &str = "close-proximity-test-credential";

pub(super) type TestWallet<C> = Wallet<
    UpdatingConfigurationRepository<LocalConfigurationRepository>,
    MockUpdatePolicyRepository,
    MockHardwareDatabaseStorage,
    MockHardwareAttestedKeyHolder,
    MockAccountProviderClient,
    MockIssuanceDiscovery,
    MockDisclosureClient,
    C,
    MockStatusListClient,
>;

pub(super) struct Fixture<C> {
    pub wallet: TestWallet<C>,
    pub reader: KeyPair,
    pub registration_certificate: Option<Value>,
    pub organization: attestation_data::organization::Organization,
    pub verification: Verification,
}

impl<C> Fixture<C> {
    pub async fn new(scenario: Scenario) -> Result<Self> {
        let rejection = scenario.rejection();
        // All key material is generated for this run and remains inside the test process.
        let issuer_ca = Ca::generate_issuer_mock_ca()?;
        let issuer = issuer_ca.generate_issuer_mock()?;
        let credential_status_list = StatusListJwt::sign(
            StatusListToken::builder(CREDENTIAL_STATUS_LIST_URI.parse()?, StatusList::new(1).pack())
                .build(&TimeGenerator),
            &issuer_ca.generate_issuer_status_list_mock()?,
        )
        .await?;
        let reader_ca = Ca::generate_wrpac_mock_ca()?;
        let reader = reader_ca.generate_wrpac_verifier_mock_with_crl()?;
        let registration_ca = MockRegistrationCertificateAuthority::new();
        let mut registration_payload =
            verifier_registration_certificate_payload(reader.certificate(), Query::new_mock_mdoc_pid_example());
        let organization =
            attestation_data::organization::Organization::from(
                &serde_json::from_value::<ParsedRegistrationCertificate>(registration_payload.0.clone())?,
            );
        let (registration_certificate, registration_status_list) = match rejection {
            Some(case) => {
                case.modify_payload(&mut registration_payload);
                (
                    case.certificate(&registration_ca, &registration_payload),
                    case.status_list(&registration_ca),
                )
            }
            None => (
                Some(Value::Bytes(match scenario {
                    Scenario::JwtDisclosure => registration_ca.sign_jwt(&registration_payload),
                    _ => registration_ca.sign_cwt(&registration_payload),
                })),
                Ok(registration_ca.status_list_client.0.clone()),
            ),
        };
        let account_key = SigningKey::generate();
        let holder_key = SigningKey::generate();
        let mut config: WalletConfiguration = serde_json::from_str(include_str!("../../../test-wallet-config.json"))?;
        config.expires = (Utc::now() + TimeDelta::days(1)).into();
        config.wrpac_trust_anchors = reader_ca.to_trust_anchors();
        config.wrprc_trust_anchors = registration_ca.trust_anchors;
        config.issuer_trust_anchors = issuer_ca.to_trust_anchors();
        config.account_server.certificate_public_keys = HashMap::from([(
            account_key.kid().try_into()?,
            CertificatePublicKey {
                key: (*account_key.verifying_key()).into(),
                used_from: Utc::now().into(),
            },
        )]);
        config.account_server.instruction_result_public_keys =
            HashMap::from([(account_key.kid().try_into()?, (*account_key.verifying_key()).into())]);

        let mut preview = PreviewableCredentialPayload::nl_pid_mdoc_example(&TimeGenerator);
        preview.attributes.overwrite(
            [PID_MDOC_DOCTYPE, PID_GIVEN_NAME],
            Attribute::Text(EXPECTED_GIVEN_NAME.to_owned()),
        );
        let (issuer_signed, mso) = CredentialPayload::from_previewable_credential_payload_unvalidated(
            preview,
            Utc::now(),
            &PublicKeyRef::from(holder_key.verifying_key()),
            None,
            Some(StatusListClaim {
                idx: 0,
                uri: CREDENTIAL_STATUS_LIST_URI.parse()?,
            }),
        )?
        .into_signed_mdoc(&issuer)
        .await?;
        let expiration = Some((&mso.validity_info.valid_until).try_into()?);
        let not_before = Some((&mso.validity_info.valid_from).try_into()?);
        let mdoc = Mdoc::new_unverified(mso, issuer_signed);
        let metadata = CredentialMetadata::new_mdoc_example(
            PID_MDOC_DOCTYPE,
            &[
                PID_GIVEN_NAME,
                PID_FAMILY_NAME,
                PID_BIRTH_DATE,
                PID_AGE_OVER_18,
                PID_BSN,
                PID_RECOVERY_CODE,
            ],
        );
        let issuer_registration = IssuerRegistration::new_mock();
        let presentation = AttestationPresentation::create_from_mdoc(
            AttestationIdentity::Ephemeral,
            PID_MDOC_DOCTYPE.to_owned(),
            metadata.clone(),
            Box::new(issuer_registration.organization().clone()),
            AttestationValidity {
                revocation_status: None,
                validity_window: ValidityWindow {
                    valid_from: not_before.map(Into::into),
                    valid_until: expiration.map(Into::into),
                },
            },
            mdoc.issuer_signed().clone().into_entries_by_namespace(),
            &config.pid_attributes,
        )?;
        let credential = CredentialWithMetadata::new(
            IssuedCredentialCopies::Mdoc(vec_nonempty![MdocCopy {
                key_identifier: HOLDER_KEY_ID.to_owned(),
                mdoc,
            }]),
            PID_MDOC_DOCTYPE.to_owned(),
            expiration,
            not_before,
            Vec::<String>::new(),
            IssuedCredentialMetadata::CredentialMetadata(metadata),
            issuer_registration,
        );
        let mut storage = MockHardwareDatabaseStorage::open_in_memory().await;
        storage
            .insert_credentials(Utc::now(), vec![(credential, presentation)])
            .await?;
        let key_holder = MockHardwareAttestedKeyHolder::generate_apple(
            AttestationEnvironment::Development,
            AppIdentifier::new_mock(),
        );
        let (attested_key, attested_key_identifier) = key_holder.random_key();
        let hw_pubkey = match &attested_key {
            AttestedKey::Apple(key) => *key.verifying_key(),
            AttestedKey::Google(key) => *key.verifying_key(),
        };
        let wallet_id = "close-proximity-device-test".to_owned();
        let certificate = SignedJwt::sign_with_sub_and_kid(
            WalletCertificateClaims {
                wallet_id: wallet_id.clone(),
                hw_pubkey: hw_pubkey.into(),
                pin_pubkey_hash: vec![0; 32],
                version: 0,
                iss: "device-test".to_owned(),
                iat: Utc::now(),
                exp: Utc::now() + TimeDelta::days(30),
            },
            &account_key,
        )
        .await?;
        let registration = RegistrationData {
            attested_key_identifier,
            pin_salt: new_pin_salt(),
            wallet_id,
            wallet_certificate: certificate.into(),
            revocation_code: RevocationCode::new_random(),
        };
        storage.insert_data(&registration).await?;
        let verification = Verification {
            issuer_trust_anchors: issuer_ca.to_trust_anchors(),
            session_transcript_hex: String::new(),
            credential_status_list,
            expected_error_status: rejection.map(RegistrationCertificateCase::response_status),
        };
        let mut wallet = Wallet::new(
            storage,
            key_holder,
            WalletRepositories {
                config_repository: UpdatingConfigurationRepository::new(
                    LocalConfigurationRepository::new(config),
                    default_config_server_config().into(),
                )
                .await,
                update_policy_repository: MockUpdatePolicyRepository::default(),
            },
            WalletClients::new_mock(MockCertificateCrlVerifier::new_for_ca(&reader_ca)),
            RegistrationStatus::Unregistered,
            None,
        );
        wallet.registration = WalletRegistration::Registered {
            attested_key: Arc::new(attested_key),
            data: registration,
        };
        wallet.lock.unlock();
        Arc::get_mut(&mut wallet.status_list_client)
            .expect("fixture owns status-list client")
            .expect_fetch()
            .with(eq(STATUS_LIST_URI.parse::<url::Url>()?))
            .times(0..=1)
            .return_once(move |_| registration_status_list);
        let account_client = Arc::get_mut(&mut wallet.account_provider_client).expect("fixture owns account client");
        // Rejected requests must never reach the account provider or use the holder signing key.
        let max_signing_calls = usize::from(rejection.is_none());
        account_client
            .expect_instruction_challenge()
            .times(0..=max_signing_calls)
            .returning(|_, _| Ok(vec![42; 32]));
        account_client
            .expect_instruction()
            // An earlier validation failure should retain its error instead of panicking when the mock is dropped.
            .times(0..=max_signing_calls)
            .return_once(move |_, instruction: Instruction<Sign>| {
                // The provider is controlled test infrastructure. Production instruction construction and
                // result verification still run; provider-side PIN/attestation verification is outside this test.
                let request = instruction.instruction.dangerous_parse_unverified().unwrap();
                assert_eq!(request.challenge, vec![42; 32]);
                assert_eq!(request.payload.messages_with_identifiers.len(), 1);
                let signatures = request
                    .payload
                    .messages_with_identifiers
                    .iter()
                    .map(|(digest, identifiers)| {
                        assert_eq!(identifiers, &[HOLDER_KEY_ID.to_owned()]);
                        let signature: Signature = holder_key.sign_prehash(digest).unwrap();
                        vec![DerSignature::from(signature)]
                    })
                    .collect();
                Ok(SignedJwt::sign_with_sub_and_kid(
                    InstructionResultClaims {
                        result: SignResult { signatures },
                        iss: "device-test".to_owned(),
                        iat: Utc::now(),
                    },
                    &account_key,
                )
                .now_or_never()
                .unwrap()
                .unwrap()
                .into())
            });

        Ok(Self {
            wallet,
            reader,
            registration_certificate,
            organization,
            verification,
        })
    }
}
