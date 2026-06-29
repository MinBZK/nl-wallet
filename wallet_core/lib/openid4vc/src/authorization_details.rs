use std::ops::Not;
use std::str::FromStr;

use attestation_types::claim_path::ClaimPath;
use derive_more::AsRef;
use derive_more::Display;
use derive_more::Into;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use serde_with::skip_serializing_none;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_at_least::VecNonEmptyUnique;

use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::issuer_metadata::CredentialConfigurationId;

// Type aliases for the `authorization_details` field as contained in the Authorization Request and Token Request.
pub type WalletAuthorizationDetails = AuthorizationDetails<VciAuthorizationDetailsEntry>;
pub type WalletAuthorizationDetailsEntries = VecNonEmpty<AuthorizationDetailsEntry<VciAuthorizationDetailsEntry>>;

// Type aliases for the `authorization_details` field as contained in the Token Response.
pub type IssuerAuthorizationDetails = AuthorizationDetails<VciIdentifierAuthorizationDetailsEntry>;
pub type IssuerAuthorizationDetailsEntries =
    VecNonEmpty<AuthorizationDetailsEntry<VciIdentifierAuthorizationDetailsEntry>>;

#[derive(Debug, thiserror::Error)]
pub enum AuthorizationDetailsError {
    #[error("none of the authorization details entries are of type openid_credential")]
    NoOpenidCredentialTypes,

    #[error("duplicate credential_configuration_id in authorization details: {}", .0.iter().join(", "))]
    DuplicateCredentialConfigIds(Vec<CredentialConfigurationId>),
}

/// This represents a list of `authorization_details` entries with the following guarantuees:
///
/// - There is at least one entry.
/// - There is at least one `openid_credential` entry.
/// - All of the `credential_configuration_id` values of the `openid_credential` entries are unique.
#[derive(Debug, Clone, AsRef, Into)]
pub struct AuthorizationDetails<T>(VecNonEmpty<AuthorizationDetailsEntry<T>>);

impl<T> AuthorizationDetails<T>
where
    T: CredentialEntry,
{
    pub fn try_new(auth_details: VecNonEmpty<AuthorizationDetailsEntry<T>>) -> Result<Self, AuthorizationDetailsError> {
        let counts_by_credential_id = auth_details
            .iter()
            .filter_map(|entry| match &entry.typed_entry {
                TypedAuthorizationDetailsEntry::OpenidCredential(vci_entry) => Some(vci_entry),
                TypedAuthorizationDetailsEntry::Other { .. } => None,
            })
            .counts_by(|vci_entry| vci_entry.credential_config_id());

        if counts_by_credential_id.is_empty() {
            return Err(AuthorizationDetailsError::NoOpenidCredentialTypes);
        }

        let duplicate_credential_ids = counts_by_credential_id
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(credential_id, _)| credential_id.clone())
            .collect_vec();

        if !duplicate_credential_ids.is_empty() {
            return Err(AuthorizationDetailsError::DuplicateCredentialConfigIds(
                duplicate_credential_ids,
            ));
        }

        Ok(Self(auth_details))
    }
}

impl<T> TryFrom<VecNonEmpty<AuthorizationDetailsEntry<T>>> for AuthorizationDetails<T>
where
    T: CredentialEntry,
{
    type Error = AuthorizationDetailsError;

    fn try_from(value: VecNonEmpty<AuthorizationDetailsEntry<T>>) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

/// The data structure for an `authorization_details` entry, based on what is defined in Section 2 of RFC9396. Any other
/// fields defined in RFC9396 besides `locations` are not used in OpenID4VCI 1.0 are therefor omitted.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthorizationDetailsEntry<T> {
    #[serde(flatten)]
    pub typed_entry: TypedAuthorizationDetailsEntry<T>,

    /// If the Credential Issuer metadata contains an authorization_servers parameter, the authorization detail's
    /// locations common data field MUST be set to the Credential Issuer Identifier value.
    pub locations: Option<VecNonEmpty<IssuerIdentifier>>,
}

/// A newtype around [`String`] that is anything but `openid_credential`, for use in [`TypedAuthorizationDetailsEntry`].
#[derive(Debug, Clone, AsRef, Display, SerializeDisplay, DeserializeFromStr)]
#[as_ref(str)]
pub struct NotOpenidCredentialType(String);

impl NotOpenidCredentialType {
    fn try_new(entry_type: String) -> Result<Self, String> {
        if entry_type == "openid_credential" {
            Err(entry_type)
        } else {
            Ok(Self(entry_type))
        }
    }
}

impl FromStr for NotOpenidCredentialType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s.to_string())
    }
}

/// An enum wrapper around both the `type` field of `authorization_details` and the custom fields provided by that type.
/// Note that only the `openid_credential` type is supported. Other types are relegated to the `Other` variant, which
/// captures the `type` field but discards any other custom fields on deserialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TypedAuthorizationDetailsEntry<T> {
    OpenidCredential(T),

    #[serde(untagged)]
    Other {
        // Capture any `type` field in `authorization_details` that is explicitly not `openid_credential`. This
        // workaround is needed in order to prevent any `VciIdentifierAuthorizationDetailsEntry` or
        // `VciAuthorizationDetailsEntry` value that fails to parse from being deserialized to this enum variant.
        #[serde(rename = "type")]
        entry_type: NotOpenidCredentialType,
    },
}

pub trait CredentialEntry {
    fn credential_config_id(&self) -> &CredentialConfigurationId;
}

/// The custom OpenID4VCI fields of `authorization_details` the Issuer may include in the Token Response. This is a
/// superset of [`VciAuthorizationDetailsEntry`] that includes the `credential_identifiers` field in order to uniquely
/// identify credential instances.
///
/// Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-6.2>
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VciIdentifierAuthorizationDetailsEntry {
    #[serde(flatten)]
    pub vci_entry: VciAuthorizationDetailsEntry,

    /// A non-empty array of strings, each uniquely identifying a Credential Dataset that can be issued using the
    /// Access Token returned in this response. Each of these Credential Datasets corresponds to the Credential
    /// Configuration referenced in the credential_configuration_id parameter. The Wallet MUST use these identifiers
    /// together with an Access Token in subsequent Credential Requests.
    pub credential_identifiers: VecNonEmptyUnique<String>,
}

impl CredentialEntry for VciIdentifierAuthorizationDetailsEntry {
    fn credential_config_id(&self) -> &CredentialConfigurationId {
        &self.vci_entry.credential_configuration_id
    }
}

/// The custom OpenID4VCI fields of `authorization_details` the Wallet may include in both the Authorization Request
/// and Token Request.
///
/// Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-5.1.1>
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VciAuthorizationDetailsEntry {
    /// String specifying a unique identifier of the Credential being described in the
    /// credential_configurations_supported map in the Credential Issuer Metadata.
    pub credential_configuration_id: CredentialConfigurationId,

    /// A non-empty array of claims description objects.
    pub claims: Option<VecNonEmpty<VciAuthorizationDetailsClaim>>,
}

impl CredentialEntry for VciAuthorizationDetailsEntry {
    fn credential_config_id(&self) -> &CredentialConfigurationId {
        &self.credential_configuration_id
    }
}

/// A claims description object as used in authorization details is an object that defines the requirements for the
/// claims that the Wallet requests to be included in the Credential.
///
/// Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-B.1>
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VciAuthorizationDetailsClaim {
    /// Non-empty array. Claim path pointer as defined in Appendix C to identify the claim(s) in the Credential.
    pub path: VecNonEmpty<ClaimPath>,

    /// Boolean which, when set to true, indicates that the Wallet will only accept a Credential that includes this
    /// claim. If set to false, the claim is not required to be included in the Credential. If the mandatory parameter
    /// is omitted, the default value is false.
    #[serde(default = "bool_value::<false>", skip_serializing_if = "<&bool>::not")]
    pub mandatory: bool,
}

const fn bool_value<const B: bool>() -> bool {
    B
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use itertools::Itertools;
    use serde_json::json;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use super::AuthorizationDetails;
    use super::AuthorizationDetailsEntry;
    use super::AuthorizationDetailsError;
    use super::TypedAuthorizationDetailsEntry;
    use super::VciAuthorizationDetailsEntry;
    use super::VciIdentifierAuthorizationDetailsEntry;

    fn other_type_auth_details_example_json() -> serde_json::Value {
        // Source: https://datatracker.ietf.org/doc/html/rfc9396#figure-10>
        json!({
            "type": "account_information",
            "actions": [
                "list_accounts"
            ],
            "locations": [
                "https://example.com/accounts"
            ]
        })
    }

    fn auth_request_auth_details_example_json() -> serde_json::Value {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-5.1.1-5>
        json!({
            "type": "openid_credential",
            "credential_configuration_id": "UniversityDegreeCredential"
        })
    }

    fn token_response_auth_details_example_json() -> serde_json::Value {
        json!({
            "type": "openid_credential",
            "credential_configuration_id": "UniversityDegreeCredential",
            "credential_identifiers": [
                "CivilEngineeringDegree-2023",
                "ElectricalEngineeringDegree-2023"
            ]
        })
    }

    fn auth_details_example_with_claims_json() -> serde_json::Value {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-A.2.3-1>
        json!({
            "type": "openid_credential",
            "credential_configuration_id": "org.iso.18013.5.1.mDL",
            "claims": [
                {"path": ["org.iso.18013.5.1","given_name"]},
                {"path": ["org.iso.18013.5.1","family_name"]},
                {"path": ["org.iso.18013.5.1","birth_date"]},
                {"path": ["org.iso.18013.5.1.aamva","organ_donor"]}
            ]
        })
    }

    #[test]
    fn test_authorization_details_deserialize_other_type() {
        let auth_details = serde_json::from_value::<AuthorizationDetailsEntry<VciAuthorizationDetailsEntry>>(
            other_type_auth_details_example_json(),
        )
        .expect("deserializing AuthorizationDetailsEntry should succeed");

        assert_eq!(
            auth_details.locations,
            Some(vec_nonempty!["https://example.com/accounts".parse().unwrap()])
        );

        let TypedAuthorizationDetailsEntry::Other { entry_type } = &auth_details.typed_entry else {
            panic!("type of AuthorizationDetailsEntry should be Other");
        };

        assert_eq!(entry_type.as_ref(), "account_information");
    }

    #[test]
    fn test_wallet_authorization_details_deserialize() {
        let auth_details = serde_json::from_value::<AuthorizationDetailsEntry<VciAuthorizationDetailsEntry>>(
            auth_request_auth_details_example_json(),
        )
        .expect("deserializing AuthorizationDetailsEntry should succeed");

        let TypedAuthorizationDetailsEntry::OpenidCredential(vci_entry) = auth_details.typed_entry else {
            panic!("type of AuthorizationDetailsEntry should be OpenidCredential");
        };

        assert_eq!(
            vci_entry.credential_configuration_id.as_ref(),
            "UniversityDegreeCredential"
        );
        assert!(vci_entry.claims.is_none());

        // Deserializing as `VciIdentifierAuthorizationDetailsEntry` should fail, as it requires
        // `credential_identifiers`.
        let _ = serde_json::from_value::<AuthorizationDetailsEntry<VciIdentifierAuthorizationDetailsEntry>>(
            auth_request_auth_details_example_json(),
        )
        .expect_err("deserializing AuthorizationDetailsEntry should fail");
    }

    #[test]
    fn test_issuer_authorization_details_deserialize() {
        let auth_details = serde_json::from_value::<AuthorizationDetailsEntry<VciIdentifierAuthorizationDetailsEntry>>(
            token_response_auth_details_example_json(),
        )
        .expect("deserializing AuthorizationDetailsEntry should succeed");

        let TypedAuthorizationDetailsEntry::OpenidCredential(vci_id_entry) = auth_details.typed_entry else {
            panic!("type of AuthorizationDetailsEntry should be OpenidCredential");
        };

        assert_eq!(
            vci_id_entry.vci_entry.credential_configuration_id.as_ref(),
            "UniversityDegreeCredential"
        );
        assert!(vci_id_entry.vci_entry.claims.is_none());
        assert_eq!(
            vci_id_entry
                .credential_identifiers
                .iter()
                .map(String::as_str)
                .collect_vec(),
            vec!["CivilEngineeringDegree-2023", "ElectricalEngineeringDegree-2023"]
        );
    }

    #[test]
    fn test_authorization_details_with_claims_deserialize() {
        let auth_details = serde_json::from_value::<AuthorizationDetailsEntry<VciAuthorizationDetailsEntry>>(
            auth_details_example_with_claims_json(),
        )
        .expect("deserializing AuthorizationDetailsEntry should succeed");

        let TypedAuthorizationDetailsEntry::OpenidCredential(vci_entry) = auth_details.typed_entry else {
            panic!("type of AuthorizationDetailsEntry should be OpenidCredential");
        };

        assert_eq!(vci_entry.credential_configuration_id.as_ref(), "org.iso.18013.5.1.mDL");
        assert_eq!(
            vci_entry
                .claims
                .as_ref()
                .map(|claims| claims.len().get())
                .unwrap_or_default(),
            4
        );

        for claim in vci_entry.claims.as_ref().unwrap() {
            assert_eq!(claim.path.len().get(), 2);

            for path in &claim.path {
                assert!(path.try_key_path().is_some());
            }
        }
    }

    #[test]
    fn test_authorization_details_deserialize_ok() {
        let json = json!([
            auth_request_auth_details_example_json(),
            other_type_auth_details_example_json(),
            auth_details_example_with_claims_json()
        ]);

        let auth_details =
            serde_json::from_value::<VecNonEmpty<AuthorizationDetailsEntry<VciAuthorizationDetailsEntry>>>(json)
                .expect("deserializing Vec of AuthorizationDetailsEntry should succeed");

        let auth_details =
            AuthorizationDetails::try_new(auth_details).expect("creating AuthorizationDetails should succeed");

        assert_eq!(auth_details.as_ref().len().get(), 3);
    }

    #[test]
    fn test_authorization_details_deserialize_error_no_openid_credential_types() {
        let json = json!([
            other_type_auth_details_example_json(),
            other_type_auth_details_example_json()
        ]);

        let auth_details =
            serde_json::from_value::<VecNonEmpty<AuthorizationDetailsEntry<VciAuthorizationDetailsEntry>>>(json)
                .expect("deserializing Vec of AuthorizationDetailsEntry should succeed");

        let error = AuthorizationDetails::try_new(auth_details).expect_err("creating AuthorizationDetails should fail");

        assert_matches!(error, AuthorizationDetailsError::NoOpenidCredentialTypes);
    }

    #[test]
    fn test_authorization_details_deserialize_error_duplicate_credential_config_ids() {
        let json = json!([
            auth_request_auth_details_example_json(),
            auth_details_example_with_claims_json(),
            auth_request_auth_details_example_json()
        ]);

        let auth_details =
            serde_json::from_value::<VecNonEmpty<AuthorizationDetailsEntry<VciAuthorizationDetailsEntry>>>(json)
                .expect("deserializing Vec of AuthorizationDetailsEntry should succeed");

        let error = AuthorizationDetails::try_new(auth_details).expect_err("creating AuthorizationDetails should fail");

        assert_matches!(
            error,
            AuthorizationDetailsError::DuplicateCredentialConfigIds(duplicate_ids)
                if duplicate_ids == vec!["UniversityDegreeCredential".to_string().into()]
        );
    }
}
