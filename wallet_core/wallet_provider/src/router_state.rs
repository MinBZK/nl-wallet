use std::error::Error;

use android_attest::root_public_key::RootPublicKey;
use audit_log::model::DbAuditLog;
use audit_log::repository::PostgresAuditLogRepository;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use crypto::PublicKey;
use crypto::keys::EcdsaKey;
use crypto::server_keys::KeyPair;
use derive_more::AsRef;
use derive_more::Into;
use futures::future::try_join_all;
use futures::try_join;
use hsm::keys::HsmEcdsaKey;
use hsm::model::Hsm;
use hsm::service::HsmError;
use hsm::service::Pkcs11Hsm;
use serde::Serialize;
use serde::de::DeserializeOwned;
use status_lists::postgres::PostgresStatusListService;
use tokio::task::AbortHandle;
use tracing::info;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use utils::generator::UuidV4AndTimeGenerator;
use uuid::Uuid;
use wallet_account::NL_WALLET_CLIENT_ID;
use wallet_account::messages::instructions::HwSignedInstruction;
use wallet_account::messages::instructions::Instruction;
use wallet_account::messages::instructions::InstructionAndResult;
use wallet_account::messages::instructions::InstructionResultMessage;
use wallet_provider_domain::keys::Kid;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::repositories::Repositories;
use wallet_provider_service::account_server::AccountServer;
use wallet_provider_service::account_server::AccountServerKeys;
use wallet_provider_service::account_server::AndroidAttestationConfiguration;
use wallet_provider_service::account_server::AppleAttestationConfiguration;
use wallet_provider_service::account_server::CurrentCertificateSigningKey;
use wallet_provider_service::account_server::PreviousCertificateSigningKey;
use wallet_provider_service::account_server::UserState;
use wallet_provider_service::flags::WalletRepoFlags;
use wallet_provider_service::instructions::HandleInstruction;
use wallet_provider_service::instructions::PinChecks;
use wallet_provider_service::instructions::ValidateInstruction;
use wallet_provider_service::keys::InstructionResultSigning;
use wallet_provider_service::keys::WalletCertificateSigning;
use wallet_provider_service::keys::certificate_signing_key_identifier;
use wallet_provider_service::pin_policy::PinPolicy;
use wallet_provider_service::status_list::KeyStorageStatusList;
use wallet_provider_service::wia_issuer::WIA_ATTESTATION_TYPE_IDENTIFIER;
use wallet_provider_service::wia_issuer::WiaIssuer;
use wscd::payload::wia::WiaWalletInfo;

use crate::errors::WalletProviderError;
use crate::router::KEY_STORAGE_TSL_PATH;
use crate::settings::Settings;

type ProductionUserState = UserState<
    Repositories,
    WalletRepoFlags<Repositories>,
    Pkcs11Hsm,
    HsmEcdsaKey,
    PostgresStatusListService<HsmEcdsaKey, WalletRepoFlags<Repositories>>,
>;

pub struct RouterState<GRC, PIC> {
    pub account_server: AccountServer<GRC, PIC>,
    pub audit_log: DbAuditLog<PostgresAuditLogRepository, UuidV7Generator>,
    pub pin_policy: PinPolicy,
    pub instruction_result_signing_key: InstructionResultSigning,
    pub certificate_signing_key: WalletCertificateSigning,
    pub user_state: ProductionUserState,
    pub max_transfer_upload_size_in_bytes: usize,
    pub wallet_transfer_concurrency_limit: usize,
    status_list_refresh_task: AbortHandle,
}

impl<GRC, PIC> Drop for RouterState<GRC, PIC> {
    fn drop(&mut self) {
        self.status_list_refresh_task.abort();
    }
}

async fn get_certificate_public_key(kid: &Kid, wallet_user_hsm: &Pkcs11Hsm) -> Result<PublicKey, HsmError> {
    wallet_user_hsm
        .get_verifying_key(&certificate_signing_key_identifier(kid))
        .await
        .map(|k| k.into())
}

async fn kid_and_previous_certificate_signing_key(
    kid: Kid,
    exp: DateTime<Utc>,
    wallet_user_hsm: &Pkcs11Hsm,
) -> Result<(Kid, PreviousCertificateSigningKey), HsmError> {
    let certificate_public_key = get_certificate_public_key(&kid, wallet_user_hsm).await?;

    Ok((
        kid,
        PreviousCertificateSigningKey {
            certificate_public_key,
            exp,
        },
    ))
}

const INSTRUCTION_RESULT_SIGNING_KEY_PREFIX: &str = "instruction_result_signing_";

#[derive(Debug, Clone, Into, AsRef)]
struct InstructionResultSigningKeyIdentifier(String);

impl InstructionResultSigningKeyIdentifier {
    pub fn from_kid(kid: &Kid) -> Self {
        Self(format!("{}{}", INSTRUCTION_RESULT_SIGNING_KEY_PREFIX, kid.as_ref()))
    }
}

impl<GRC, PIC> RouterState<GRC, PIC> {
    pub async fn new_from_settings(
        settings: Settings,
        wallet_user_hsm: Pkcs11Hsm,
        google_crl_client: GRC,
        play_integrity_client: PIC,
    ) -> Result<RouterState<GRC, PIC>, Box<dyn Error>> {
        let wc_signing_key_identifier = certificate_signing_key_identifier(&settings.current_certificate_kid);

        let certificate_signing_key = WalletCertificateSigning {
            kid: settings.current_certificate_kid.clone(),
            key: HsmEcdsaKey::new(wc_signing_key_identifier, wallet_user_hsm.clone()),
        };
        let instruction_result_signing_key = InstructionResultSigning {
            kid: settings.current_instruction_result_kid.clone(),
            key: HsmEcdsaKey::new(
                InstructionResultSigningKeyIdentifier::from_kid(&settings.current_instruction_result_kid).into(),
                wallet_user_hsm.clone(),
            ),
        };

        let certificate_public_key = certificate_signing_key.verifying_key().await?.into();

        let apple_trust_anchors = settings
            .ios
            .root_certificates
            .into_iter()
            .map(|anchor| anchor.to_owned_trust_anchor())
            .collect();
        let apple_config = AppleAttestationConfiguration::new(
            settings.ios.team_identifier,
            settings.ios.bundle_identifier,
            settings.ios.environment.into(),
            apple_trust_anchors,
        );

        let android_installation_method = settings.android.installation_method();
        let android_root_public_keys = settings
            .android
            .root_public_keys
            .into_iter()
            .map(RootPublicKey::from)
            .collect();
        let android_config = AndroidAttestationConfiguration {
            root_public_keys: android_root_public_keys,
            package_name: settings.android.package_name,
            installation_method: android_installation_method,
            certificate_hashes: settings.android.play_store_certificate_hashes,
        };

        let previous_certificate_signing_keys = try_join_all(
            settings
                .previous_certificate_kids
                .unwrap_or_default()
                .into_iter()
                .map(async |(kid, exp)| kid_and_previous_certificate_signing_key(kid, exp, &wallet_user_hsm).await),
        )
        .await?
        .into_iter()
        .collect();

        let account_server = AccountServer::new(
            "account_server".into(),
            settings.instruction_challenge_timeout,
            settings.wallet_certificate_validity,
            AccountServerKeys {
                current_certificate_signing_key: CurrentCertificateSigningKey {
                    kid: settings.current_certificate_kid,
                    public_key: certificate_public_key,
                },
                previous_certificate_signing_keys,
                pin_pubkey_encryption_kids: settings.pin_pubkey_encryption_kid,
                revocation_code_key_identifier: settings.revocation_code_key_identifier,
            },
            settings.recovery_code_paths.into(),
            apple_config,
            android_config,
            google_crl_client,
            play_integrity_client,
        );

        // TODO refactor wallet_provider_database to generic database module to share with issuance server (PVW-5196)
        let db = Db::new(settings.database.url, settings.database.options).await?;

        let audit_db = Db::new_connection(settings.audit_log.url, settings.audit_log.options).await?;

        let audit_log = DbAuditLog {
            repository: PostgresAuditLogRepository {
                db_connection: audit_db,
            },
            time_generator: TimeGenerator,
            uuid_generator: UuidV7Generator,
        };

        let repositories = Repositories::from(db.clone());
        let flags = WalletRepoFlags::try_new(repositories.clone(), settings.flags_refresh_delay).await?;
        flags.start_refresh_job();
        let wia_status_list = PostgresStatusListService::try_new(
            WIA_ATTESTATION_TYPE_IDENTIFIER,
            db.to_connection(),
            settings
                .wia_settings
                .wia_status_list
                .into_config(wallet_user_hsm.clone())
                .await?,
            flags.clone(),
        )
        .await?;
        wia_status_list.initialize_lists().await?;
        let status_list_refresh_task = wia_status_list.start_refresh_job();

        let pin_policy = PinPolicy::new(
            settings.pin_policy.rounds,
            settings.pin_policy.attempts_per_round,
            settings
                .pin_policy
                .timeouts
                .into_iter()
                .map(Duration::from_std)
                .collect::<Result<_, _>>()?,
        );

        let (wia_key_pair, key_attestation_key_pair, key_storage_tsl_key_pair) = try_join!(
            KeyPair::new(
                HsmEcdsaKey::new(
                    settings.wia_settings.wia_signing_key_identifier,
                    wallet_user_hsm.clone(),
                ),
                settings.wia_settings.wia_certificate,
            ),
            KeyPair::new(
                HsmEcdsaKey::new(
                    settings.key_attestation_settings.key_attestation_signing_key_identifier,
                    wallet_user_hsm.clone(),
                ),
                settings.key_attestation_settings.key_attestation_certificate,
            ),
            KeyPair::new(
                HsmEcdsaKey::new(
                    settings.key_attestation_settings.key_storage_status_list.key_identifier,
                    wallet_user_hsm.clone(),
                ),
                settings
                    .key_attestation_settings
                    .key_storage_status_list
                    .key_certificate
            )
        )?;

        let wia_issuer = WiaIssuer::new(
            wia_key_pair,
            NL_WALLET_CLIENT_ID.to_string(),
            WiaWalletInfo {
                wallet_name: settings.wia_settings.wia_wallet_name,
                wallet_link: settings.wia_settings.wia_wallet_link,
                wallet_version: settings.wia_settings.wia_wallet_version,
                wallet_solution_certification_information: settings
                    .wia_settings
                    .wia_wallet_solution_certification_information,
            },
        );

        let key_storage_tsl_public_uri = settings.public_url.join(KEY_STORAGE_TSL_PATH);
        let key_storage_status_list = KeyStorageStatusList::try_new(
            key_storage_tsl_public_uri,
            settings.key_attestation_settings.key_storage_status_list.expiry,
            settings.key_attestation_settings.key_storage_status_list.ttl,
            key_storage_tsl_key_pair,
            flags.clone(),
        )?;

        let state = RouterState {
            account_server,
            audit_log,
            instruction_result_signing_key,
            certificate_signing_key,
            pin_policy,
            max_transfer_upload_size_in_bytes: settings.max_transfer_upload_size_in_bytes,
            wallet_transfer_concurrency_limit: settings.wallet_transfer_concurrency_limit,
            user_state: UserState {
                repositories,
                flags,
                wallet_user_hsm,
                wia_issuer,
                key_attestation_key_pair,
                key_attestation_certification: settings.key_attestation_settings.key_attestation_certification,
                wia_status_tracking_validity: settings.wia_settings.wia_status_tracking_validity,
                key_storage_status_tracking_validity: settings
                    .key_attestation_settings
                    .key_storage_status_list
                    .tracking_validity,
                attestation_wrapping_kids: settings.attestation_wrapping_kid,
                pid_issuer_trust_anchors: settings.pid_issuer_trust_anchors,
                wia_status_list,
                key_storage_status_list,
            },
            status_list_refresh_task,
        };

        Ok(state)
    }

    pub async fn handle_instruction<I, R>(
        &self,
        instruction: Instruction<I>,
    ) -> Result<InstructionResultMessage<R>, WalletProviderError>
    where
        I: InstructionAndResult<Result = R> + HandleInstruction<Result = R> + ValidateInstruction + PinChecks,
        R: Serialize + DeserializeOwned,
    {
        let result = self
            .account_server
            .handle_instruction(
                instruction,
                &self.instruction_result_signing_key,
                &UuidV4AndTimeGenerator,
                &self.pin_policy,
                &self.user_state,
            )
            .await?;

        info!("Replying with the instruction result");

        Ok(InstructionResultMessage { result })
    }

    pub async fn handle_hw_signed_instruction<I, R>(
        &self,
        instruction: HwSignedInstruction<I>,
    ) -> Result<InstructionResultMessage<R>, WalletProviderError>
    where
        I: InstructionAndResult<Result = R> + HandleInstruction<Result = R> + ValidateInstruction,
        R: Serialize + DeserializeOwned,
    {
        let result = self
            .account_server
            .handle_hw_signed_instruction(
                instruction,
                &self.instruction_result_signing_key,
                &UuidV4AndTimeGenerator,
                &self.user_state,
            )
            .await?;

        info!("Replying with the instruction result");

        Ok(InstructionResultMessage { result })
    }
}

// Time-Sortable and Index-Friendly Uuid generator
#[derive(Clone, Copy)]
pub struct UuidV7Generator;

impl Generator<Uuid> for UuidV7Generator {
    fn generate(&self) -> Uuid {
        Uuid::now_v7()
    }
}
