use std::sync::Arc;

use anyhow::Result;
use attestation_data::attributes::Attribute;
use attestation_data::credential_payload::CredentialContents;
use attestation_data::credential_payload::CredentialPayload;
use attestation_data::registration_certificate::ParsedRegistrationCertificate;
use attestation_data::registration_certificate::mock::MockRegistrationCertificateAuthority;
use attestation_data::registration_certificate::mock::STATUS_LIST_URI;
use attestation_data::registration_certificate::mock::verifier_registration_certificate_payload;
use attestation_data::validity::ValidityWindow;
use attestation_types::pid_constants::PID_GIVEN_NAME;
use attestation_types::pid_constants::PID_MDOC_DOCTYPE;
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
use mdoc::holder::Mdoc;
use mockall::predicate::always;
use mockall::predicate::eq;
use openid4vc::wallet_issuance::credential::CredentialWithMetadata;
use openid4vc::wallet_issuance::credential::IssuedCredentialCopies;
use openid4vc::wallet_issuance::credential::IssuedCredentialMetadata;
use openid4vc::wallet_issuance::credential::MdocCopy;
use openid4vc::wallet_issuance::issuer_registration::IssuerRegistration;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::signature::hazmat::PrehashSigner;
use p256::elliptic_curve::Generate;
use token_status_list::status_list::StatusList;
use token_status_list::status_list_jwt::StatusListJwt;
use token_status_list::status_list_token::StatusListToken;
use utils::generator::TimeGenerator;
use utils::vec_nonempty;
use wallet_account::messages::instructions::Instruction;
use wallet_account::messages::instructions::Sign;
use wallet_account::messages::instructions::SignResult;

use super::CREDENTIAL_STATUS_LIST_URI;
use super::EXPECTED_GIVEN_NAME;
use super::RegistrationCertificateCase;
use super::Scenario;
use super::Verification;
use crate::AttestationIdentity;
use crate::AttestationPresentation;
use crate::attestation::AttestationValidity;
use crate::config::LocalConfigurationRepository;
use crate::config::UpdatingConfigurationRepository;
use crate::storage::Storage;
use crate::wallet::test::AccountServerKeys;
use crate::wallet::test::TestWalletInMemoryStorage;
use crate::wallet::test::WalletDeviceVendor;
use crate::wallet::test::create_wallet_configuration_with_keys;
use crate::wallet::test::create_wp_result_with_key;
use crate::wallet::test::nl_pid_mdoc_credential_metadata_example;

const HOLDER_KEY_ID: &str = "close-proximity-test-credential";

pub(super) type TestWallet<C> =
    TestWalletInMemoryStorage<UpdatingConfigurationRepository<LocalConfigurationRepository>, C>;

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
        let account_keys = AccountServerKeys::generate();
        let holder_key = SigningKey::generate();
        let mut config = create_wallet_configuration_with_keys(&account_keys);
        config.expires = (Utc::now() + TimeDelta::days(1)).into();
        config.wrpac_trust_anchors = reader_ca.to_trust_anchors();
        config.wrprc_trust_anchors = registration_ca.trust_anchors;
        config.issuer_trust_anchors = issuer_ca.to_trust_anchors();

        let mut preview = CredentialContents::nl_pid_mdoc_example(&TimeGenerator);
        preview.attributes.overwrite(
            [PID_MDOC_DOCTYPE, PID_GIVEN_NAME],
            Attribute::Text(EXPECTED_GIVEN_NAME.to_owned()),
        );
        let (issuer_signed, mso) = CredentialPayload::from_contents_unvalidated(
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
        let mdoc = Mdoc::new_unverified(mso, issuer_signed);
        let contents = CredentialContents::from_mdoc(mdoc.clone())?;
        let metadata = nl_pid_mdoc_credential_metadata_example();
        let issuer_registration = IssuerRegistration::new_mock();
        let presentation = AttestationPresentation::create_from_mdoc(
            AttestationIdentity::Ephemeral,
            PID_MDOC_DOCTYPE.to_owned(),
            metadata.clone(),
            Box::new(issuer_registration.organization().clone()),
            AttestationValidity {
                revocation_status: None,
                validity_window: ValidityWindow {
                    valid_from: contents.not_before.map(Into::into),
                    valid_until: contents.expires.map(Into::into),
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
            contents,
            Vec::<String>::new(),
            IssuedCredentialMetadata::CredentialMetadata(metadata),
            issuer_registration,
        );
        let mut wallet = TestWallet::new_registered_and_unlocked_with_deps(
            WalletDeviceVendor::Apple,
            config,
            &account_keys.certificate_signing_key,
            MockCertificateCrlVerifier::new_for_ca(&reader_ca),
        )
        .await;
        wallet
            .mut_storage()
            .insert_credentials(Utc::now(), vec![(credential, presentation)])
            .await?;
        let verification = Verification {
            issuer_trust_anchors: issuer_ca.to_trust_anchors(),
            session_transcript_hex: String::new(),
            credential_status_list,
            expected_error_status: rejection.map(RegistrationCertificateCase::response_status),
        };
        Arc::get_mut(&mut wallet.status_list_client)
            .expect("fixture owns status-list client")
            .expect_fetch()
            .with(eq(STATUS_LIST_URI.parse::<url::Url>()?), always())
            .times(0..=1)
            .return_once(move |_, _| registration_status_list.map(Into::into));
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
                Ok(create_wp_result_with_key(
                    SignResult { signatures },
                    &account_keys.instruction_result_signing_key,
                ))
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
