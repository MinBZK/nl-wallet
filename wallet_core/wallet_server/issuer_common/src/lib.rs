use openid4vc::issuer::IssuanceData;
use openid4vc::issuer::Issuer;
use server_utils::keys::PrivateKeyVariant;
use server_utils::store::SessionStoreVariant;
use status_lists::postgres::NoRevokeAll;
use status_lists::postgres::PostgresStatusListService;

use crate::nonce_store::ProofNonceStore;

mod entity;
pub mod nonce_store;
pub mod par_store;
pub mod settings;
pub mod state_bridge_store;

pub type IssuanceServerIssuer = Issuer<
    PrivateKeyVariant,
    PostgresStatusListService<PrivateKeyVariant, NoRevokeAll>,
    SessionStoreVariant<IssuanceData>,
    ProofNonceStore,
>;
