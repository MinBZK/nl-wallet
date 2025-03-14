use chrono::serde::ts_seconds;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

/// JWT claims of a PoP (Proof of Possession). Used a.o. as a JWT proof in a Credential Request
/// (<https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#section-7.2.1.1>).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwtPopClaims {
    pub iss: String,
    pub aud: String,
    pub nonce: Option<String>,
    #[serde(with = "ts_seconds")]
    pub iat: DateTime<Utc>,
}

impl JwtPopClaims {
    pub fn new(nonce: Option<String>, iss: String, aud: String) -> Self {
        Self {
            nonce,
            iss,
            aud,
            iat: Utc::now(),
        }
    }
}
