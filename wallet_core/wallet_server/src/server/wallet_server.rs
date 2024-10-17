use anyhow::Result;
use p256::{ecdsa::VerifyingKey, pkcs8::DecodePublicKey};

use openid4vc::{issuer::AttributeService, server_state::SessionStore, verifier::DisclosureData};

use super::*;
use crate::{
    issuer::{create_issuance_router, IssuerKeyRing},
    settings::Settings,
    verifier,
};

pub async fn serve<A, DS, IS>(
    attr_service: A,
    settings: Settings,
    disclosure_sessions: DS,
    issuance_sessions: IS,
) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    DS: SessionStore<DisclosureData> + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
{
    let log_requests = settings.log_requests;

    let private_keys: IssuerKeyRing<_> = settings.issuer.private_keys.try_into()?;
    let wte_privkey = VerifyingKey::from_public_key_der(&settings.issuer.wte_issuer_pubkey)?;
    let wallet_issuance_router = create_issuance_router(
        &settings.urls,
        private_keys,
        issuance_sessions,
        attr_service,
        settings.issuer.wallet_client_ids,
        wte_privkey,
    )?;
    let (wallet_disclosure_router, requester_router) =
        verifier::create_routers(settings.urls, settings.verifier, disclosure_sessions)?;

    listen(
        settings.wallet_server,
        settings.requester_server,
        Router::new()
            .nest("/issuance", wallet_issuance_router)
            .nest("/disclosure", wallet_disclosure_router),
        Router::new().nest("/disclosure", requester_router),
        log_requests,
    )
    .await
}
