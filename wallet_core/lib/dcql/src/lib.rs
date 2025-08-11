pub mod normalized;

use std::ops::Not;

use itertools::Itertools;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

use attestation_types::claim_path::ClaimPath;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_at_least::VecNonEmptyUnique;

trait MayHaveUniqueId {
    fn id(&self) -> Option<&str>;
}

#[nutype(
    derive(Debug, Clone, PartialEq, Eq, AsRef, TryFrom, IntoIterator, Serialize, Deserialize),
    validate(predicate = |items| !items.is_empty() && items.iter().flat_map(MayHaveUniqueId::id).all_unique()),
)]
pub struct UniqueIdVec<T: MayHaveUniqueId>(Vec<T>);

/// A DCQL query, encoding constraints on the combinations of credentials and claims that are requested.
/// The Wallet must evaluate the query against the Credentials it holds and returns Presentations matching the query.
///
/// <https://openid.net/specs/openid-4-verifiable-presentations-1_0-28.html#name-digital-credentials-query-l>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    /// Credential Queries that specify the requested Credentials.
    pub credentials: UniqueIdVec<CredentialQuery>,

    /// Additional constraints, if any, on which of the requested Credentials to return.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub credential_sets: Vec<CredentialSetQuery>,
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
    pub id: String,

    /// Specifies the format of the requested Credential.
    #[serde(flatten)]
    pub format: CredentialQueryFormat,

    /// Indicates whether multiple Credentials can be returned for this Credential Query.
    /// If omitted, the default value is false.
    #[serde(default = "bool_value::<false>", skip_serializing_if = "<&bool>::not")]
    pub multiple: bool,

    /// Expected authorities or trust frameworks that certify Issuers, if any, that the Verifier will accept.
    /// Every Credential returned by the Wallet SHOULD match at least one of the conditions present
    /// in the corresponding trusted_authorities array if present.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trusted_authorities: Vec<TrustedAuthoritiesQuery>,

    /// Indicates whether the Verifier requires a Cryptographic Holder Binding proof. The default value is true,
    /// i.e., a Verifiable Presentation with Cryptographic Holder Binding is required.
    /// If set to false, the Verifier accepts a Credential without Cryptographic Holder Binding proof.
    #[serde(default = "bool_value::<true>", skip_serializing_if = "Clone::clone")]
    pub require_cryptographic_holder_binding: bool,

    #[serde(flatten)]
    pub claims_selection: ClaimsSelection,
}

impl MayHaveUniqueId for CredentialQuery {
    fn id(&self) -> Option<&str> {
        Some(self.id.as_str())
    }
}

/// Specifies which claims (if any) of the Credential is requested by the RP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClaimsSelection {
    /// The RP requests none of the selectively disclosable claims of the Credential.
    NoSelectivelyDisclosable,

    /// The RP specifies several options of combinations of requested claims.
    Combinations {
        /// Objects that specify claims in the requested Credential.
        claims: UniqueIdVec<ClaimsQuery>,

        /// Arrays of identifiers for elements in claims that specifies which combinations of claims for the Credential
        /// are requested.
        claim_sets: VecNonEmpty<VecNonEmptyUnique<String>>,
    },

    /// The RP requests all of the contained claims.
    All {
        /// Objects that specify claims in the requested Credential.
        claims: UniqueIdVec<ClaimsQuery>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
pub struct CredentialSetQuery {
    /// A non-empty array, where each value in the array is a list of Credential Query identifiers representing
    /// one set of Credentials that satisfies the use case. The value of each element in the options array is
    /// an array of identifiers which reference elements in the `credentials` field of [`Query`].
    pub options: VecNonEmpty<VecNonEmptyUnique<String>>,

    /// Indicates whether this set of Credentials is required to satisfy the particular use case at the Verifier.
    /// If omitted, the default value is true.
    #[serde(default = "bool_value::<true>", skip_serializing_if = "Clone::clone")]
    pub required: bool,
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

    // Allow parsing of methods not supported by this implementation.
    #[serde(untagged)]
    Other(String),
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
    pub id: Option<String>,

    /// Claims path pointers that specify the path to a claim within the Credential.
    pub path: VecNonEmpty<ClaimPath>,

    /// Expected values of the claim, if any. If the values property is present, the Wallet SHOULD return the claim
    /// only if the type and value of the claim both match exactly for at least one of the elements in the array.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<serde_json::Value>,

    /// Whether the RP intends to retain the attribute after disclosure for some amount of time.
    /// Note: this flag is specific to the mdoc attestation format and should not be present in case of other formats.
    ///
    /// <https://openid.net/specs/openid-4-verifiable-presentations-1_0-28.html#name-parameter-in-the-claims-que>
    pub intent_to_retain: Option<bool>,
}

impl MayHaveUniqueId for ClaimsQuery {
    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}

const fn bool_value<const B: bool>() -> bool {
    B
}

#[cfg(any(test, feature = "examples"))]
pub mod examples {
    use super::Query;

    pub(crate) const MULTIPLE_CREDENTIALS_DCQL_QUERY_BYTES: &[u8] =
        include_bytes!("../examples/spec/multiple_credentials_dcql_query.json");
    pub(crate) const WITH_CREDENTIAL_SETS_DCQL_QUERY_BYTES: &[u8] =
        include_bytes!("../examples/spec/with_credential_sets_dcql_query.json");
    pub(crate) const WITH_CLAIM_SETS_DCQL_QUERY_BYTES: &[u8] =
        include_bytes!("../examples/spec/with_claim_sets_dcql_query.json");
    pub(crate) const WITH_VALUES_DCQL_QUERY_BYTES: &[u8] =
        include_bytes!("../examples/spec/with_values_dcql_query.json");

    impl Query {
        fn from_slice(slice: &[u8]) -> Self {
            serde_json::from_slice::<Query>(slice).unwrap()
        }

        pub fn example_with_multiple_credentials() -> Self {
            Self::from_slice(MULTIPLE_CREDENTIALS_DCQL_QUERY_BYTES)
        }

        pub fn example_with_credential_sets() -> Self {
            Self::from_slice(WITH_CREDENTIAL_SETS_DCQL_QUERY_BYTES)
        }

        pub fn example_with_claim_sets() -> Self {
            Self::from_slice(WITH_CLAIM_SETS_DCQL_QUERY_BYTES)
        }

        pub fn example_with_values() -> Self {
            Self::from_slice(WITH_VALUES_DCQL_QUERY_BYTES)
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::Query;
    use super::examples::*;

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
                    "claims": [
                        { "path": [ "org.iso.7367.1", "vehicle_holder" ], "intent_to_retain": true },
                        { "path": [ "org.iso.18013.5.1", "first_name" ], "values": [ "John" ] }
                    ]
                }
            ],
            "credential_sets": [
                {
                    "options": [ [ "pid" ], [ "other_pid" ] ]
                },
                {
                    "options": [ [ "mdl" ] ],
                    "required": false
                }
            ]
        });

        assert_eq!(serde_json::to_value(&query).unwrap(), expected);
    }

    #[test]
    fn test_empty_credentials_error() {
        let json = json!({
            "credentials": []
        });

        let _ = serde_json::from_value::<Query>(json).expect_err("deserializing Query should not succeed");
    }

    #[test]
    fn test_empty_claims_error() {
        let json = json!({
            "credentials": [
                {
                    "id": "pid",
                    "format": "dc+sd-jwt",
                    "meta": {
                        "vct_values": [ "https://credentials.example.com/identity_credential" ]
                    },
                    "claims": []
                }
            ]
        });

        let _ = serde_json::from_value::<Query>(json).expect_err("deserializing Query should not succeed");
    }

    #[rstest]
    #[case(("foo", "bar"), (Some("bleh"), Some("blah")), true)]
    #[case(("foo", "bar"), (None, None), true)]
    #[case(("foo", "bar"), (Some("bleh"), None), true)]
    #[case(("foo", "bar"), (None, Some("bleh")), true)]
    #[case(("foo", "foo"), (None, None), false)]
    #[case(("foo", "foo"), (Some("bleh"), Some("blah")), false)]
    #[case(("foo", "bar"), (Some("bleh"), Some("bleh")), false)]
    fn test_duplicate_id_errors(
        #[case] (first_credential_id, second_credential_id): (&str, &str),
        #[case] (first_claim_id, second_claim_id): (Option<&str>, Option<&str>),
        #[case] should_succeed: bool,
    ) {
        let json = json!({
            "credentials": [
                {
                    "id": first_credential_id,
                    "format": "dc+sd-jwt",
                    "meta": {
                        "vct_values": ["https://credentials.example.com/identity_credential" ]
                    },
                    "claims": [
                        { "id": first_claim_id, "path": [ "given_name" ] },
                        { "id": second_claim_id, "path": [ "family_name" ] }
                    ]
                },
                {
                    "id": second_credential_id,
                    "format": "dc+sd-jwt",
                    "meta": {
                        "vct_values": [ "https://othercredentials.example/pid" ]
                    },
                    "require_cryptographic_holder_binding": false,
                    "claims": [
                        { "path": [ "address", null, 1, "street_address" ] }
                    ]
                }
            ]
        });

        let result = serde_json::from_value::<Query>(json);

        if should_succeed {
            let _ = result.expect("deserializing Query should succeed");
        } else {
            let _ = result.expect_err("deserializing Query should not succeed");
        }
    }
}
