use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use futures::future::try_join_all;
use tokio::net::TcpListener;
use tracing::info;

use crypto::trust_anchor::BorrowingTrustAnchor;
use hsm::service::Pkcs11Hsm;
use openid4vc::issuer::IssuanceData;
use openid4vc::issuer::WteConfig;
use openid4vc::server_state::MemoryWteTracker;
use openid4vc::server_state::SessionStore;
use openid4vc::verifier::DisclosureData;
use openid4vc::verifier::SessionTypeReturnUrl;
use openid4vc::verifier::UseCase;
use openid4vc_server::issuer::create_issuance_router;
use openid4vc_server::verifier::create_wallet_router;
use openid4vc_server::verifier::RequestUriBehaviour;
use openid4vc_server::verifier::VerifierConfig;
use server_utils::keys::PrivateKeyVariant;
use server_utils::server::create_wallet_listener;
use server_utils::server::decorate_router;
use wallet_common::built_info::version_string;

use crate::disclosure::AttributesFetcher;
use crate::disclosure::DisclosureBasedAttributeService;
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

    let result_handler = IssuanceResultHandler {
        issuance_sessions: Arc::clone(&issuance_sessions),
        credential_issuer: issuer_settings.server_settings.public_url.join_base_url("issuance/"),
        attributes_fetcher,
    };

    let use_cases = try_join_all(settings.disclosure_settings.into_iter().map(|(id, s)| async {
        Ok::<_, anyhow::Error>((
            id.clone(),
            UseCase::try_new(
                id,
                s.key_pair.parse(hsm.clone()).await?,
                SessionTypeReturnUrl::Both,
                Some(s.to_disclose),
                Some("openid-credential-offer://".parse().unwrap()),
            )?,
        ))
    }))
    .await?
    .into_iter()
    .collect::<HashMap<String, UseCase<PrivateKeyVariant>>>()
    .into();

    let attr_service = DisclosureBasedAttributeService::new(Arc::clone(&issuance_sessions));

    let issuance_router = create_issuance_router(
        &issuer_settings.server_settings.public_url,
        attestation_config,
        Arc::clone(&issuance_sessions),
        attr_service,
        issuer_settings.wallet_client_ids,
        Option::<WteConfig<MemoryWteTracker>>::None, // The compiler forces us to explicitly specify a type here
    );

    let disclosure_router = create_wallet_router(
        VerifierConfig {
            public_url: issuer_settings.server_settings.public_url.join_base_url("disclosure"),
            universal_link_base_url: settings.universal_link_base_url.clone(),
            ephemeral_id_secret: None,
            use_cases,
            issuer_trust_anchors: issuer_settings
                .server_settings
                .issuer_trust_anchors
                .iter()
                .map(BorrowingTrustAnchor::to_owned_trust_anchor)
                .collect(),
            result_handler,
            sessions: disclosure_sessions,
        },
        &RequestUriBehaviour::ByUsecaseId,
    );

    listen(
        listener,
        Router::new()
            .nest("/issuance", issuance_router)
            .nest("/disclosure", disclosure_router),
        log_requests,
    )
    .await
}

async fn listen(wallet_listener: TcpListener, mut wallet_router: Router, log_requests: bool) -> Result<()> {
    wallet_router = decorate_router(wallet_router, log_requests);

    info!("{}", version_string());
    info!("listening for wallet on {}", wallet_listener.local_addr()?);

    axum::serve(wallet_listener, wallet_router)
        .await
        .expect("wallet server should be started");

    Ok(())
}
