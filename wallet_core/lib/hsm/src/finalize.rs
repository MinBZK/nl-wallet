//! Separate finalize module to prevent guard and pool from being directly accessed
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use cryptoki::context::CInitializeArgs;
use cryptoki::context::CInitializeFlags;
use cryptoki::context::Pkcs11;
use cryptoki::types::AuthPin;
use r2d2_cryptoki::Pool;
use r2d2_cryptoki::SessionAuth;
use r2d2_cryptoki::SessionManager;
use r2d2_cryptoki::r2d2::Builder;
use r2d2_cryptoki::r2d2::LoggingErrorHandler;

use crate::service::HsmError;

/// Guard to finalize the PKCS#11 context.
///
/// Since `cryptoki` 0.12.0, `Pkcs11` no longer finalizes itself on `Drop`, so it has to be finalized explicitly here
/// instead. This must happen only once when every session has been closed. If `C_Finalize` is never called, or is
/// called while sessions are still open, some HSMs (e.g. SoftHSM linked against Botan) may crash during process
/// exit while tearing down state they believe is still in use.
struct Pkcs11FinalizeGuard(Pkcs11);

impl Pkcs11FinalizeGuard {
    pub fn new(library_path: PathBuf) -> Result<Self, cryptoki::error::Error> {
        let pkcs11 = Pkcs11::new(library_path)?;
        pkcs11.initialize(CInitializeArgs::new(CInitializeFlags::OS_LOCKING_OK))?;
        Ok(Self(pkcs11))
    }
}

impl Drop for Pkcs11FinalizeGuard {
    fn drop(&mut self) {
        if let Err(error) = self.0.clone().finalize() {
            tracing::warn!("failed to finalize PKCS#11 context: {error}");
        }
    }
}

/// Combined `Pool` and `Pkcs11FinalizeGuard` to ensure the Pool is closed before the PKCS#11 context is finalized.
#[derive(Clone)]
pub struct Pkcs11FinalizePool {
    pool: Pool,

    // Keep a handle to `Pkcs11FinalizeGuard` to finalize the PKCS#11 context once the pool is dropped.
    // Declared after `pool` so that all pooled sessions are closed (via `Pool`'s drop mechanism)
    // before the PKCS#11 context is finalized.
    #[expect(dead_code, reason = "only ever dropped, never read")]
    guard: Arc<Pkcs11FinalizeGuard>,
}

impl Pkcs11FinalizePool {
    pub fn new<F>(library_path: PathBuf, user_pin: String, adapt_builder: F) -> Result<Self, HsmError>
    where
        F: FnOnce(Builder<SessionManager>) -> Builder<SessionManager>,
    {
        let pkcs11 = Pkcs11FinalizeGuard::new(library_path)?;

        let slot = *pkcs11
            .0
            .get_slots_with_initialized_token()?
            .first()
            .ok_or(HsmError::NoInitializedSlotAvailable)?;

        let session_auth = SessionAuth::RwUser(AuthPin::from(user_pin));
        let manager = SessionManager::new(pkcs11.0.clone(), slot, &session_auth);

        let builder = Pool::builder();
        let pool = adapt_builder(builder)
            .connection_customizer(session_auth.into_customizer())
            .error_handler(Box::new(LoggingErrorHandler))
            .build(manager)?;

        Ok(Self {
            pool,
            guard: Arc::new(pkcs11),
        })
    }
}

impl Deref for Pkcs11FinalizePool {
    type Target = Pool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}
