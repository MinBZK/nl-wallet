use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use futures::future::try_join_all;
use server_utils::server::add_cache_layer;
use server_utils::server::secure_internal_router;
use tokio::net::TcpListener;

use crypto::trust_anchor::BorrowingTrustAnchor;
use hsm::service::Pkcs11Hsm;
use openid4vc::credential::OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME;
use openid4vc::issuer::IssuanceData;
use openid4vc::issuer::Issuer;
use openid4vc::issuer::TrivialAttributeService;
use openid4vc::issuer::WuaConfig;
use openid4vc::server_state::SessionStore;
use openid4vc::verifier::DisclosureData;
use openid4vc::verifier::SessionTypeReturnUrl;
use openid4vc::verifier::WalletInitiatedUseCase;
use openid4vc::verifier::WalletInitiatedUseCases;
use openid4vc_server::issuer::create_issuance_router;
use openid4vc_server::verifier::VerifierFactory;
use server_utils::keys::PrivateKeyVariant;
use server_utils::server::create_internal_listener;
use server_utils::server::create_wallet_listener;
use server_utils::server::listen;
use status_lists::revoke::create_revocation_router;
use token_status_list::status_list_service::StatusListRevocationService;
use token_status_list::status_list_service::StatusListServices;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::verifier::RevocationVerifier;

use crate::disclosure::AttributesFetcher;
use crate::disclosure::IssuanceResultHandler;
use crate::settings::IssuanceServerSettings;

#[expect(clippy::too_many_arguments, reason = "Setup function")]
pub async fn serve<A, L, IS, DS, C>(
    settings: IssuanceServerSettings,
    hsm: Option<Pkcs11Hsm>,
    issuance_sessions: Arc<IS>,
    disclosure_sessions: Arc<DS>,
    attributes_fetcher: A,
    status_list_services: L,
    status_list_router: Option<Router>,
    status_list_client: C,
) -> Result<()>
where
    IS: SessionStore<IssuanceData> + Send + Sync + 'static,
    DS: SessionStore<DisclosureData> + Send + Sync + 'static,
    A: AttributesFetcher + Sync + 'static,
    L: StatusListServices + StatusListRevocationService + Sync + 'static,
    C: StatusListClient + Sync + 'static,
{
    serve_with_listeners(
        create_wallet_listener(&settings.issuer_settings.server_settings.wallet_server).await?,
        create_internal_listener(&settings.issuer_settings.server_settings.internal_server).await?,
        settings,
        hsm,
        issuance_sessions,
        disclosure_sessions,
        attributes_fetcher,
        status_list_services,
        status_list_router,
        status_list_client,
    )
    .await
}

#[expect(clippy::too_many_arguments, reason = "Setup function")]
pub async fn serve_with_listeners<A, L, IS, DS, C>(
    wallet_listener: TcpListener,
    internal_listener: Option<TcpListener>,
    settings: IssuanceServerSettings,
    hsm: Option<Pkcs11Hsm>,
    issuance_sessions: Arc<IS>,
    disclosure_sessions: Arc<DS>,
    attributes_fetcher: A,
    status_list_services: L,
    status_list_router: Option<Router>,
    status_list_client: C,
) -> Result<()>
where
    IS: SessionStore<IssuanceData> + Send + Sync + 'static,
    DS: SessionStore<DisclosureData> + Send + Sync + 'static,
    A: AttributesFetcher + Sync + 'static,
    L: StatusListServices + StatusListRevocationService + Sync + 'static,
    C: StatusListClient + Sync + 'static,
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
                s.dcql_query.try_into()?,
                format!("{OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME}://").parse().unwrap(),
            )?,
        ))
    }))
    .await?
    .into_iter()
    .collect::<HashMap<String, WalletInitiatedUseCase<PrivateKeyVariant>>>();

    let use_cases = WalletInitiatedUseCases::new(use_cases);

    let status_list_services = Arc::new(status_list_services);
    let issuer = Arc::new(Issuer::new(
        issuance_sessions,
        TrivialAttributeService,
        attestation_config,
        &issuer_settings.server_settings.public_url,
        issuer_settings.wallet_client_ids.clone(),
        Option::<WuaConfig>::None, // The compiler forces us to explicitly specify a type here,
        Arc::clone(&status_list_services),
    ));

    let issuance_router = create_issuance_router(Arc::clone(&issuer));

    let result_handler = IssuanceResultHandler {
        issuer,
        credential_issuer: issuer_settings.server_settings.public_url.join_base_url("issuance/"),
        attributes_fetcher,
    };

    let revocation_verifier = RevocationVerifier::new(status_list_client);

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
        settings.extending_vct_values.unwrap_or_default(),
    )
    .create_wallet_router(disclosure_sessions, revocation_verifier, Some(Box::new(result_handler)));

    let mut wallet_router = Router::new()
        .nest("/issuance", add_cache_layer(issuance_router))
        .nest("/disclosure", add_cache_layer(disclosure_router));

    if let Some(status_list_router) = status_list_router {
        wallet_router = wallet_router.merge(status_list_router);
    }

    let mut internal_router = create_revocation_router(status_list_services);
    internal_router = secure_internal_router(&issuer_settings.server_settings.internal_server, internal_router);
    internal_router = add_cache_layer(internal_router);
    listen(
        wallet_listener,
        internal_listener,
        wallet_router,
        internal_router,
        log_requests,
    )
    .await
}
