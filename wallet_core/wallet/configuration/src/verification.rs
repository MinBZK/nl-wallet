use std::sync::LazyLock;
use std::time::Duration;

use jwt::Algorithm;
use jwt::JwtValidation;

/// Leeway allowed when evaluating the `exp` of a [`WalletConfiguration`], in order to handle clock skew between the
/// wallet device and the configuration server.
///
/// [`WalletConfiguration`]: crate::wallet_config::WalletConfiguration
pub const CONFIG_EXPIRY_LEEWAY: Duration = Duration::from_secs(60);

pub static WALLET_CONFIG_VALIDATION: LazyLock<JwtValidation> = LazyLock::new(|| {
    let mut validation = JwtValidation::default_with_algorithms([Algorithm::ES256]);
    validation.require_exp();
    validation.set_leeway(CONFIG_EXPIRY_LEEWAY);

    validation
});
