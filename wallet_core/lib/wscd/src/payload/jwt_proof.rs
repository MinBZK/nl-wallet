use chrono::DateTime;
use chrono::Utc;
use chrono::serde::ts_seconds;
use jwt::JwtTyp;
use jwt::UnverifiedJwt;
use jwt::headers::HeaderWithJwk;
use jwt::headers::HeaderWithTyp;
use jwt::nonce::Nonce;
use serde::Deserialize;
use serde::Serialize;
use utils::generator::Generator;

pub const OPENID4VCI_PROOF_JWT_TYP: &str = "openid4vci-proof+jwt";

pub type JwtProof = UnverifiedJwt<JwtProofClaims, JwtProofHeader>;

/// The JOSE header for the OpenID4VCI JWT Proof Type.
///
/// Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-F.1>
///
/// This contains the following fields:
/// - The `alg` and `typ` header are provided by `HeaderWithTyp`.
/// - The `jwk` header is provided by `HeaderWithJwk`.
///
/// Note that we choose for the Wallet to always send the `jwk` header. Since the issuer need not be fully
/// interoperable, we can ignore the `x5c`, `kid` and `trust_chain` fields.
///
/// TODO (PVW-6238): Include `key_attestation` header field.
pub type JwtProofHeader = HeaderWithJwk<HeaderWithTyp>;

/// JWT Proof Type for a OpenID4VCI Credential Request, implemented as JWT claims of a PoP (Proof of Possession).
///
/// Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-F.1>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtProofClaims {
    /// The value of this claim MUST be the client_id of the Client making the Credential request. This claim MUST be
    /// omitted if the access token authorizing the issuance call was obtained from a Pre-Authorized Code Flow through
    /// anonymous access to the token endpoint.
    ///
    /// NB: Since HAIP requires always using client authentication at the Token Endpoint, anonymous access is not
    ///     applicable and this field should never be omitted.
    pub iss: String,

    /// The value of this claim MUST be the Credential Issuer Identifier.
    pub aud: String,

    /// The value of this claim MUST be the time at which the key proof was issued using the syntax defined in RFC7519.
    #[serde(with = "ts_seconds")]
    pub iat: DateTime<Utc>,

    /// The value type of this claim MUST be a string, where the value is a server-provided `c_nonce``. It MUST be
    /// present when the issuer has a Nonce Endpoint as defined in Section 7.
    pub nonce: Option<Nonce>,
}

impl JwtTyp for JwtProofClaims {
    const TYP: &'static str = OPENID4VCI_PROOF_JWT_TYP;
}

impl JwtProofClaims {
    pub fn new(iss: String, aud: String, nonce: Option<Nonce>, time: &impl Generator<DateTime<Utc>>) -> Self {
        Self {
            iss,
            aud,
            iat: time.generate(),
            nonce,
        }
    }
}
