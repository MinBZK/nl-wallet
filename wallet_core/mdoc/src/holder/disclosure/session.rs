use serde::{Deserialize, Serialize};

use crate::verifier::SessionType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierUrlParameters {
    pub session_type: SessionType,
}
