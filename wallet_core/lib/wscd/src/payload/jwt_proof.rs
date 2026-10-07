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
use serde_with::skip_serializing_none;
use utils::generator::Generator;

use super::key_attestation::KeyAttestation;

pub const OPENID4VCI_PROOF_JWT_TYP: &str = "openid4vci-proof+jwt";

pub type JwtProof = UnverifiedJwt<JwtProofClaims, JwtProofHeader>;

/// The JOSE header for the OpenID4VCI JWT Proof Type.
///
/// Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-F.1>
///
/// This contains the following claims:
/// - The `alg` and `typ` claim are provided by `HeaderWithTyp`.
/// - The `jwk` claim is provided by `HeaderWithJwk`.
/// - The `key_attestation` claim is the embedded `attestation` proof, in the form of a Key Attestation in JWT format.
///   Although this value is optional according to OpenID4VCI, HAIP mandates its presence.
///
/// Note that we choose for the Wallet to always send the `jwk` header. Since the issuer need not be fully
/// interoperable, we can ignore the `x5c`, `kid` and `trust_chain` fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtProofHeader {
    #[serde(flatten)]
    jwk_header: HeaderWithJwk<HeaderWithTyp>,

    key_attestation: KeyAttestation,
}

/// JWT Proof Type for a OpenID4VCI Credential Request, implemented as JWT claims of a PoP (Proof of Possession).
///
/// Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-F.1>
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtProofClaims {
    /// OPTIONAL (string). The value of this claim MUST be the client_id of the Client making the Credential request.
    /// This claim MUST be omitted if the access token authorizing the issuance call was obtained from a
    /// Pre-Authorized Code Flow through anonymous access to the token endpoint.
    ///
    /// NB: Since HAIP requires always using client authentication at the Token Endpoint, anonymous access is not
    ///     applicable and this field should never be omitted.
    pub iss: String,

    /// REQUIRED (string). The value of this claim MUST be the Credential Issuer Identifier.
    pub aud: String,

    /// REQUIRED (number). The value of this claim MUST be the time at which the key proof was issued using the syntax
    /// defined in RFC7519.
    #[serde(with = "ts_seconds")]
    pub iat: DateTime<Utc>,

    /// OPTIONAL (string). The value type of this claim MUST be a string, where the value is a server-provided
    /// `c_nonce`. It MUST be present when the issuer has a Nonce Endpoint.
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
