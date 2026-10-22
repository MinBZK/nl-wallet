use jwk_simple::Key;
use serde::Deserialize;
use serde::Serialize;
use utils::vec_at_least::VecNonEmpty;

/// A JSON Web Key Set as defined in [RFC7517], that contains at least one key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: VecNonEmpty<Key>,
}
