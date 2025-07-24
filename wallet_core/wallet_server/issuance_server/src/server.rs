use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use futures::future::try_join_all;
use tokio::net::TcpListener;

use crypto::trust_anchor::BorrowingTrustAnchor;
use hsm::service::Pkcs11Hsm;
use openid4vc::credential::OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME;
use openid4vc::issuer::IssuanceData;
use openid4vc::issuer::Issuer;
use openid4vc::issuer::TrivialAttributeService;
use openid4vc::issuer::WteConfig;
use openid4vc::server_state::MemoryWteTracker;
use openid4vc::server_state::SessionStore;
use openid4vc::verifier::DisclosureData;
use openid4vc::verifier::SessionTypeReturnUrl;
use openid4vc::verifier::WalletInitiatedUseCase;
use openid4vc::verifier::WalletInitiatedUseCases;
use openid4vc_server::issuer::create_issuance_router;
use openid4vc_server::verifier::VerifierFactory;
use server_utils::keys::PrivateKeyVariant;
use server_utils::server::create_wallet_listener;
use server_utils::server::listen;

use crate::disclosure::AttributesFetcher;
use crate::disclosure::IssuanceResultHandler;
use crate::settings::IssuanceServerSettings;

pub async fn serve<A, IS, DS>(
    settings: IssuanceServerSettings,
    hsm: Option<Pkcs11Hsm>,
    issuance_sessions: Arc<IS>,
    disclosure_sessions: Arc<DS>,
    attributes_fetcher: A,
) -> Result<()>
where
    IS: SessionStore<IssuanceData> + Send + Sync + 'static,
    DS: SessionStore<DisclosureData> + Send + Sync + 'static,
    A: AttributesFetcher + Sync + 'static,
{
    serve_with_listener(
        create_wallet_listener(&settings.issuer_settings.server_settings.wallet_server).await?,
        settings,
        hsm,
        issuance_sessions,
        disclosure_sessions,
        attributes_fetcher,
    )
    .await
}

pub async fn serve_with_listener<A, IS, DS>(
    listener: TcpListener,
    settings: IssuanceServerSettings,
    hsm: Option<Pkcs11Hsm>,
    issuance_sessions: Arc<IS>,
    disclosure_sessions: Arc<DS>,
    attributes_fetcher: A,
) -> Result<()>
where
    IS: SessionStore<IssuanceData> + Send + Sync + 'static,
    DS: SessionStore<DisclosureData> + Send + Sync + 'static,
    A: AttributesFetcher + Sync + 'static,
{
    let log_requests = settings.issuer_settings.server_settings.log_requests;
    let issuer_settings = settings.issuer_settings;
    let type_metadata = issuer_settings.metadata;
    let attestation_config = issuer_settings.attestation_settings.parse(&hsm, &type_metadata).await?;

    let use_cases = try_join_all(settings.disclosure_settings.into_iter().map(|(id, s)| async {
        Ok::<_, anyhow::Error>((
            id,
            WalletInitiatedUseCase::try_new(
                s.key_pair.parse(hsm.clone()).await?,
                SessionTypeReturnUrl::Both,
                s.dcql_query,
                format!("{OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME}://").parse().unwrap(),
            )?,
        ))
    }))
    .await?
    .into_iter()
    .collect::<HashMap<String, WalletInitiatedUseCase<PrivateKeyVariant>>>();

    let use_cases = WalletInitiatedUseCases::new(use_cases);

    let issuer = Arc::new(Issuer::new(
        issuance_sessions,
        TrivialAttributeService,
        attestation_config,
        &issuer_settings.server_settings.public_url,
        issuer_settings.wallet_client_ids.clone(),
        Option::<WteConfig<MemoryWteTracker>>::None, // The compiler forces us to explicitly specify a type here
    ));

    let issuance_router = create_issuance_router(Arc::clone(&issuer));

    let result_handler = IssuanceResultHandler {
        issuer,
        credential_issuer: issuer_settings.server_settings.public_url.join_base_url("issuance/"),
        attributes_fetcher,
    };

    let disclosure_router = VerifierFactory::new(
        issuer_settings.server_settings.public_url.join_base_url("disclosure"),
        settings.universal_link_base_url,
        use_cases,
        issuer_settings
            .server_settings
            .issuer_trust_anchors
            .iter()
            .map(BorrowingTrustAnchor::to_owned_trust_anchor)
            .collect(),
        issuer_settings.wallet_client_ids,
    )
    .create_wallet_router(disclosure_sessions, Some(Box::new(result_handler)));

    listen(
        listener,
        Router::new()
            .nest("/issuance", issuance_router)
            .nest("/disclosure", disclosure_router),
        log_requests,
    )
    .await
}
