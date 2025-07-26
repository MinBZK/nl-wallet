use std::error::Error;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tracing::info;
use uuid::Uuid;

use android_attest::root_public_key::RootPublicKey;
use crypto::keys::EcdsaKey;
use hsm::keys::HsmEcdsaKey;
use hsm::service::Pkcs11Hsm;
use utils::generator::Generator;
use wallet_account::messages::instructions::Instruction;
use wallet_account::messages::instructions::InstructionAndResult;
use wallet_account::messages::instructions::InstructionResultMessage;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::repositories::Repositories;
use wallet_provider_service::account_server::AccountServer;
use wallet_provider_service::account_server::AccountServerKeys;
use wallet_provider_service::account_server::AndroidAttestationConfiguration;
use wallet_provider_service::account_server::AppleAttestationConfiguration;
use wallet_provider_service::account_server::UserState;
use wallet_provider_service::instructions::HandleInstruction;
use wallet_provider_service::instructions::ValidateInstruction;
use wallet_provider_service::keys::InstructionResultSigning;
use wallet_provider_service::keys::WalletCertificateSigning;
use wallet_provider_service::pin_policy::PinPolicy;
use wallet_provider_service::wte_issuer::HsmWteIssuer;

use crate::errors::WalletProviderError;
use crate::settings::Settings;

pub struct RouterState<GRC, PIC> {
    pub account_server: AccountServer<GRC, PIC>,
    pub pin_policy: PinPolicy,
    pub instruction_result_signing_key: InstructionResultSigning,
    pub certificate_signing_key: WalletCertificateSigning,
    pub user_state: UserState<Repositories, Pkcs11Hsm, HsmWteIssuer<Pkcs11Hsm>>,
}

impl<GRC, PIC> RouterState<GRC, PIC> {
    pub async fn new_from_settings(
        settings: Settings,
        hsm: Pkcs11Hsm,
        google_crl_client: GRC,
        play_integrity_client: PIC,
    ) -> Result<RouterState<GRC, PIC>, Box<dyn Error>> {
        let certificate_signing_key = WalletCertificateSigning(HsmEcdsaKey::new(
            settings.certificate_signing_key_identifier,
            hsm.clone(),
        ));
        let instruction_result_signing_key = InstructionResultSigning(HsmEcdsaKey::new(
            settings.instruction_result_signing_key_identifier,
            hsm.clone(),
        ));

        let certificate_signing_pubkey = certificate_signing_key.verifying_key().await?;

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

        let account_server = AccountServer::new(
            "account_server".into(),
            settings.instruction_challenge_timeout,
            AccountServerKeys {
                wallet_certificate_signing_pubkey: (&certificate_signing_pubkey).into(),
                encryption_key_identifier: settings.pin_pubkey_encryption_key_identifier,
                pin_public_disclosure_protection_key_identifier: settings
                    .pin_public_disclosure_protection_key_identifier,
            },
            apple_config,
            android_config,
            google_crl_client,
            play_integrity_client,
        );

        let db = Db::new(
            settings.database.connection_string(),
            settings.database.connection_options,
        )
        .await?;

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

        let repositories = Repositories::from(db);
        let wte_issuer = HsmWteIssuer::new(
            HsmEcdsaKey::new(settings.wte_signing_key_identifier, hsm.clone()),
            settings.wte_issuer_identifier,
            hsm.clone(),
            settings.attestation_wrapping_key_identifier.clone(),
        );

        let state = RouterState {
            account_server,
            instruction_result_signing_key,
            certificate_signing_key,
            pin_policy,
            user_state: UserState {
                repositories,
                wallet_user_hsm: hsm,
                wte_issuer,
                wrapping_key_identifier: settings.attestation_wrapping_key_identifier,
            },
        };

        Ok(state)
    }

    pub async fn handle_instruction<I, R>(
        &self,
        instruction: Instruction<I>,
    ) -> Result<InstructionResultMessage<R>, WalletProviderError>
    where
        I: InstructionAndResult<Result = R> + HandleInstruction<Result = R> + ValidateInstruction,
        R: Serialize + DeserializeOwned,
    {
        let result = self
            .account_server
            .handle_instruction(
                instruction,
                &self.instruction_result_signing_key,
                self,
                &self.pin_policy,
                &self.user_state,
            )
            .await?;

        info!("Replying with the instruction result");

        Ok(InstructionResultMessage { result })
    }
}

impl<GRC, PIC> Generator<uuid::Uuid> for RouterState<GRC, PIC> {
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
}

impl<GRC, PIC> Generator<DateTime<Utc>> for RouterState<GRC, PIC> {
    fn generate(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
