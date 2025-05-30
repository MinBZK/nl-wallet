use std::fmt::Display;
use std::fmt::Formatter;

use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

use utils::vec_at_least::VecNonEmpty;

/// A DCQL query, encoding constraints on the combinations of credentials and claims that are requested.
/// The Wallet must evaluate the query against the Credentials it holds and returns Presentations matching the query.
///
/// <https://openid.net/specs/openid-4-verifiable-presentations-1_0-28.html#name-digital-credentials-query-l>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    /// Credential Queries that specify the requested Credentials.
    pub credentials: VecNonEmpty<CredentialQuery>,

    /// Additional constraints, if any, on which of the requested Credentials to return.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub credential_sets: Vec<CredentialQuerySets>,
}

/// Represents a request for a presentation of one or more matching Credentials.
///
/// <https://openid.net/specs/openid-4-verifiable-presentations-1_0-28.html#name-credential-query>
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialQuery {
    /// Identifies the Credential in the response and, if provided, the constraints in credential_sets. MUST be
    /// non-empty consisting of alphanumeric, underscore (_) or hyphen (-) characters. MUST be unique within the
    /// Authorization Request.
    id: String,

    /// Specifies the format of the requested Credential.
    #[serde(flatten)]
    format: CredentialQueryFormat,

    /// Indicates whether multiple Credentials can be returned for this Credential Query.
    /// If omitted, the default value is false.
    #[serde(default = "bool_value::<false>")]
    multiple: bool,

    /// Expected authorities or trust frameworks that certify Issuers, if any, that the Verifier will accept.
    /// Every Credential returned by the Wallet SHOULD match at least one of the conditions present
    /// in the corresponding trusted_authorities array if present.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    trusted_authorities: Vec<TrustedAuthoritiesQuery>,

    /// Indicates whether the Verifier requires a Cryptographic Holder Binding proof. The default value is true,
    /// i.e., a Verifiable Presentation with Cryptographic Holder Binding is required.
    /// If set to false, the Verifier accepts a Credential without Cryptographic Holder Binding proof.
    #[serde(default = "bool_value::<true>")]
    require_cryptographic_holder_binding: bool,

    /// Objects that specify claims in the requested Credential. Optional.
    /// Verifiers MUST NOT point to the same claim more than once in a single query.
    /// Wallets SHOULD ignore such duplicate claim queries.
    /// If empty the wallet MUST disclose none of the attributes of the Credential.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    claims: Vec<ClaimsQuery>,

    /// Arrays of identifiers for elements in claims that specifies which combinations of claims for the Credential
    /// are requested.
    /// Optional. MUST NOT be present if claims is absent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    claim_sets: Vec<VecNonEmpty<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format", content = "meta", rename_all = "snake_case")]
pub enum CredentialQueryFormat {
    MsoMdoc {
        /// Doctype of the requested Verifiable Credential
        doctype_value: String,
    },
    #[serde(rename = "dc+sd-jwt")]
    SdJwt {
        /// Allowed values for the type of the requested Verifiable Credential.
        /// All elements in the array MUST be valid type identifiers as defined in [I-D.ietf-oauth-sd-jwt-vc].
        /// The Wallet MAY return credentials that inherit from any of the specified types, following the
        /// inheritance logic defined in [I-D.ietf-oauth-sd-jwt-vc].
        ///
        /// [I-D.ietf-oauth-sd-jwt-vc]: https://datatracker.ietf.org/doc/html/draft-ietf-oauth-sd-jwt-vc-08
        vct_values: VecNonEmpty<String>,
    },
}

/// Represents a request for one or more credentials to satisfy a particular use case with the Verifier.
///
/// <https://openid.net/specs/openid-4-verifiable-presentations-1_0-28.html#name-credential-set-query>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialQuerySets {
    /// A non-empty array, where each value in the array is a list of Credential Query identifiers representing
    /// one set of Credentials that satisfies the use case. The value of each element in the options array is
    /// an array of identifiers which reference elements in the `credentials` field of [`Query`].
    options: VecNonEmpty<VecNonEmpty<String>>,

    /// Indicates whether this set of Credentials is required to satisfy the particular use case at the Verifier.
    /// If omitted, the default value is true.
    #[serde(default = "bool_value::<true>")]
    required: bool,
}

/// Information that helps to identify an authority or the trust framework that certifies Issuers.
/// A Credential is identified as a match to a Trusted Authorities Query if it matches with one of the provided values
/// in one of the provided types.
///
/// <https://openid.net/specs/openid-4-verifiable-presentations-1_0-28.html#dcql_trusted_authorities>
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "values", rename_all = "snake_case")]
pub enum TrustedAuthoritiesQuery {
    /// Contains the KeyIdentifier of the AuthorityKeyIdentifier as defined in Section 4.2.1.1 of [RFC5280],
    /// encoded as base64url. The raw byte representation of this element MUST match with the AuthorityKeyIdentifier
    /// element of an X.509 certificate in the certificate chain present in the credential (e.g., in the header of an
    /// mdoc or SD-JWT). Note that the chain can consist of a single certificate and the credential can include the
    /// entire X.509 chain or parts of it.
    Aki(VecNonEmpty<String>),
}

/// Specifies claims in the requested Credential.
///
/// <https://openid.net/specs/openid-4-verifiable-presentations-1_0-28.html#name-claims-query>
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimsQuery {
    /// A string identifying the particular claim.
    /// REQUIRED if claim_sets is present in the Credential Query; OPTIONAL otherwise.
    /// The value MUST be a non-empty string consisting of alphanumeric, underscore (_) or hyphen (-) characters.
    /// Within the particular claims array, the same id MUST NOT be present more than once.
    id: Option<String>,

    /// Claims path pointers that specify the path to a claim within the Credential.
    path: VecNonEmpty<ClaimPath>,

    /// Expected values of the claim, if any. If the values property is present, the Wallet SHOULD return the claim
    /// only if the type and value of the claim both match exactly for at least one of the elements in the array.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    values: Vec<serde_json::Value>,

    /// Whether the RP intends to retain the attribute after disclosure for some amount of time.
    /// Note: this flag is specific to the mdoc attestation format and should not be present in case of other formats.
    ///
    /// <https://openid.net/specs/openid-4-verifiable-presentations-1_0-28.html#name-parameter-in-the-claims-que>
    intent_to_retain: Option<bool>,
}

/// Element of a claims path pointer.
/// TODO: deduplicate this with the enum in the `sd_jwt_vc_metadata` crate (PVW-4421).
///
/// <https://openid.net/specs/openid-4-verifiable-presentations-1_0-28.html#name-claims-path-pointer>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClaimPath {
    /// Select a claim in an object.
    SelectByKey(String),

    /// Select all elements within an array.
    SelectAll,

    /// Select an element in an array.
    SelectByIndex(usize),
}

impl Display for ClaimPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaimPath::SelectByKey(key) => write!(f, "{}", key),
            ClaimPath::SelectAll => f.write_str("*"),
            ClaimPath::SelectByIndex(index) => write!(f, "{}", index),
        }
    }
}

const fn bool_value<const B: bool>() -> bool {
    B
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::Query;

    const MULTIPLE_CREDENTIALS_DCQL_QUERY_BYTES: &[u8] =
        include_bytes!("../examples/spec/multiple_credentials_dcql_query.json");
    const WITH_CREDENTIAL_SETS_DCQL_QUERY_BYTES: &[u8] =
        include_bytes!("../examples/spec/with_credential_sets_dcql_query.json");
    const WITH_CLAIM_SETS_DCQL_QUERY_BYTES: &[u8] = include_bytes!("../examples/spec/with_claim_sets_dcql_query.json");
    const WITH_VALUES_DCQL_QUERY_BYTES: &[u8] = include_bytes!("../examples/spec/with_values_dcql_query.json");

    #[rstest]
    #[case(MULTIPLE_CREDENTIALS_DCQL_QUERY_BYTES)]
    #[case(WITH_CREDENTIAL_SETS_DCQL_QUERY_BYTES)]
    #[case(WITH_CLAIM_SETS_DCQL_QUERY_BYTES)]
    #[case(WITH_VALUES_DCQL_QUERY_BYTES)]
    fn deserialize_spec_examples(#[case] bytes: &[u8]) {
        serde_json::from_slice::<Query>(bytes).unwrap();
    }

    /// A (contrived) example containing all supported syntactic features.
    const ALL_FEATURES_DCQL_QUERY_BYTES: &[u8] = include_bytes!("../examples/all_features_dcql_query.json");

    #[test]
    fn deserialize_example_with_all_features() {
        let query: Query = serde_json::from_slice(ALL_FEATURES_DCQL_QUERY_BYTES).unwrap();

        let expected = json!({
            "credentials": [
                {
                    "id": "pid",
                    "format": "dc+sd-jwt",
                    "meta": {
                        "vct_values": [ "https://credentials.example.com/identity_credential" ]
                    },
                    "multiple": false,
                    "require_cryptographic_holder_binding": true,
                    "claims": [
                        { "id": "a", "path": [ "given_name" ] },
                        { "id": "b", "path": [ "family_name" ] },
                        { "id": "c", "path": [ "address", "street_address" ] }
                    ],
                    "claim_sets": [
                        [ "a","c" ],
                        [ "b","c" ]
                    ]
                },
                {
                    "id": "other_pid",
                    "format": "dc+sd-jwt",
                    "meta": {
                        "vct_values": [ "https://othercredentials.example/pid" ]
                    },
                    "multiple": false,
                    "trusted_authorities": [
                        { "type": "aki", "values": [ "s9tIpPmhxdiuNkHMEWNpYim8S8Y" ] }
                    ],
                    "require_cryptographic_holder_binding": false,
                    "claims": [
                        { "path": [ "given_name" ] },
                        { "path": [ "family_name" ] },
                        { "path": [ "address", null, 1, "street_address" ] }
                    ]
                },
                {
                    "id": "mdl",
                    "format": "mso_mdoc",
                    "meta": {
                        "doctype_value": "org.iso.7367.1.mVRC"
                    },
                    "multiple": true,
                    "require_cryptographic_holder_binding": true,
                    "claims": [
                        { "path": [ "org.iso.7367.1", "vehicle_holder" ], "intent_to_retain": true },
                        { "path": [ "org.iso.18013.5.1", "first_name" ], "values": [ "John" ] }
                    ]
                }
            ],
            "credential_sets": [
                {
                    "options": [ [ "pid" ], [ "other_pid" ] ],
                    "required": true
                },
                {
                    "options": [ [ "mdl" ] ],
                    "required": false
                }
            ]
        });

        assert_eq!(serde_json::to_value(&query).unwrap(), expected);
    }
}
