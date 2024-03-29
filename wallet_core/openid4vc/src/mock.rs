use nl_wallet_mdoc::{
    holder::{MdocCopies, TrustAnchor},
    utils::keys::{KeyFactory, MdocEcdsaKey},
};
use wallet_common::config::wallet_config::BaseUrl;

use crate::{
    issuance_session::{HttpOpenidMessageClient, IssuanceSession, IssuanceSessionError},
    token::{AttestationPreview, TokenRequest},
};

// We can't use `mockall::automock!` on the `IssuerClient` trait directly since `automock` doesn't accept
// traits using generic methods, and "impl trait" arguments, so we use `mockall::mock!` to make an indirection.

mockall::mock! {
    pub IssuanceSession {
        pub fn start() -> Result<(Self, Vec<AttestationPreview>), IssuanceSessionError>
        where
            Self: Sized;

        pub fn accept(
            &self,
        ) -> Result<Vec<MdocCopies>, IssuanceSessionError>;

        pub fn reject(self) -> Result<(), IssuanceSessionError>;
    }
}

impl IssuanceSession for MockIssuanceSession {
    async fn start_issuance(
        _: HttpOpenidMessageClient,
        _: BaseUrl,
        _: TokenRequest,
        _: &[TrustAnchor<'_>],
    ) -> Result<(Self, Vec<AttestationPreview>), IssuanceSessionError>
    where
        Self: Sized,
    {
        Self::start()
    }

    async fn accept_issuance<K: MdocEcdsaKey>(
        &self,
        _: &[TrustAnchor<'_>],
        _: impl KeyFactory<Key = K>,
        _: BaseUrl,
    ) -> Result<Vec<MdocCopies>, IssuanceSessionError> {
        self.accept()
    }

    async fn reject_issuance(self) -> Result<(), IssuanceSessionError> {
        self.reject()
    }
}
