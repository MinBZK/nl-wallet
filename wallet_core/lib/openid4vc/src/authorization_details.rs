use derive_more::AsRef;
use derive_more::Into;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use strum::EnumString;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_at_least::VecNonEmptyUnique;

use crate::metadata::issuer_metadata::CredentialConfigurationId;

// Type aliases for the `authorization_details` field as contained in the Authorization Request and Token Request.
pub type WalletAuthorizationDetails = AuthorizationDetails<CredentialConfigEntry>;
pub type WalletAuthorizationDetailsEntries = VecNonEmpty<EntryContainer<CredentialConfigEntry>>;

// Type aliases for the `authorization_details` field as contained in the Token Response.
pub type IssuerAuthorizationDetails = AuthorizationDetails<CredentialEntry>;
pub type IssuerAuthorizationDetailsEntries = VecNonEmpty<EntryContainer<CredentialEntry>>;

#[derive(Debug, thiserror::Error)]
pub enum AuthorizationDetailsError {
    #[error("duplicate credential_configuration_id in authorization details: {}", .0.iter().join(", "))]
    DuplicateCredentialConfigIds(Vec<CredentialConfigurationId>),
}

/// This represents a list of `authorization_details` entries with the following guarantees:
///
/// - There is at least one entry.
/// - All entries are of the `openid_credential` type.
/// - All of the `credential_configuration_id` values of the `openid_credential` entries are unique.
#[derive(Debug, Clone, AsRef, Into)]
pub struct AuthorizationDetails<T>(VecNonEmpty<EntryContainer<T>>);

impl<T> AuthorizationDetails<T>
where
    T: EntryWithConfigId,
{
    pub fn try_new(auth_details: VecNonEmpty<EntryContainer<T>>) -> Result<Self, AuthorizationDetailsError> {
        // Note that `counts_by_credential_id` cannot be empty, as the input `auth_details` is not empty.
        let counts_by_credential_id = auth_details
            .iter()
            .counts_by(|entry_container| entry_container.entry.credential_config_id());

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

impl AuthorizationDetails<CredentialEntry> {
    pub fn from_credential_ids_and_identifiers<'a>(
        credential_ids_and_identifiers: impl IntoNonEmptyIterator<Item = (&'a CredentialConfigurationId, String)>,
    ) -> Self {
        let entries = credential_ids_and_identifiers
            .into_iter()
            .into_group_map()
            .into_iter()
            .map(|(config_id, identifiers)| {
                EntryContainer::new_credential(
                    config_id.clone(),
                    VecNonEmpty::try_from(identifiers)
                        .expect("into_group_map() values should never contain an empty Vec")
                        .into(),
                )
            })
            .collect_vec()
            .try_into()
            .expect("source iterator is non-empty and into_group_map() should never result in an empty HashMap");

        AuthorizationDetails::try_new(entries).expect(
            "all entries are created as openid_credential and into_group_map() guarantees removal of duplicates",
        )
    }
}

impl<T> TryFrom<VecNonEmpty<EntryContainer<T>>> for AuthorizationDetails<T>
where
    T: EntryWithConfigId,
{
    type Error = AuthorizationDetailsError;

    fn try_from(value: VecNonEmpty<EntryContainer<T>>) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, strum::Display, EnumString, SerializeDisplay, DeserializeFromStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum EntryType {
    #[default]
    OpenidCredential,
}

/// The data structure for an `authorization_details` entry of type `openid_credential`, based on what is defined in
/// Section 2 of RFC9396. Any other fields defined in RFC9396 are either not used in OpenID4VCI 1.0 or not supported by
/// this implementation and are therefore omitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryContainer<T> {
    #[serde(rename = "type")]
    pub entry_type: EntryType,

    #[serde(flatten)]
    pub entry: T,
    // OpenID4VCI states the following:
    //
    // "If the Credential Issuer metadata contains an authorization_servers parameter, the authorization detail's
    //  locations common data field MUST be set to the Credential Issuer Identifier value."
    //
    // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-5.1.1-7>
    //
    // However, as we choose to have the wallet never send an `authorization_details` value, we can safely omit this
    // field from the struct.
}

impl EntryContainer<CredentialConfigEntry> {
    pub fn new_credential_config(credential_configuration_id: CredentialConfigurationId) -> Self {
        Self {
            entry_type: EntryType::OpenidCredential,
            entry: CredentialConfigEntry {
                credential_configuration_id,
            },
        }
    }
}

impl EntryContainer<CredentialEntry> {
    pub fn new_credential(
        credential_configuration_id: CredentialConfigurationId,
        credential_identifiers: VecNonEmptyUnique<String>,
    ) -> Self {
        Self {
            entry_type: EntryType::OpenidCredential,
            entry: CredentialEntry {
                config_entry: CredentialConfigEntry {
                    credential_configuration_id,
                },
                credential_identifiers,
            },
        }
    }
}

pub trait EntryWithConfigId {
    fn credential_config_id(&self) -> &CredentialConfigurationId;
}

/// The custom OpenID4VCI fields of `authorization_details` the Issuer may include in the Token Response. This is a
/// superset of [`CredentialConfigEntry`] that includes the `credential_identifiers` field in order to uniquely identify
/// credential instances.
///
/// Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-6.2>
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CredentialEntry {
    #[serde(flatten)]
    pub config_entry: CredentialConfigEntry,

    /// A non-empty array of strings, each uniquely identifying a Credential Dataset that can be issued using the
    /// Access Token returned in this response. Each of these Credential Datasets corresponds to the Credential
    /// Configuration referenced in the credential_configuration_id parameter. The Wallet MUST use these identifiers
    /// together with an Access Token in subsequent Credential Requests.
    pub credential_identifiers: VecNonEmptyUnique<String>,
}

impl EntryWithConfigId for CredentialEntry {
    fn credential_config_id(&self) -> &CredentialConfigurationId {
        &self.config_entry.credential_configuration_id
    }
}

/// The custom OpenID4VCI fields of `authorization_details` the Wallet may include in both the Authorization Request
/// and Token Request.
///
/// Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-5.1.1>
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CredentialConfigEntry {
    /// String specifying a unique identifier of the Credential being described in the
    /// credential_configurations_supported map in the Credential Issuer Metadata.
    pub credential_configuration_id: CredentialConfigurationId,
    // Note that we leave out the `claims` field here. As the wallet does not support requesting specific claims for a
    // credential and the issuer does not support interpreting them, the field will never be used.
    //
    // See: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-5.1.1-3.3> and
    // <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-B.1>
}

impl EntryWithConfigId for CredentialConfigEntry {
    fn credential_config_id(&self) -> &CredentialConfigurationId {
        &self.credential_configuration_id
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use itertools::Itertools;
    use serde_json::json;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use super::AuthorizationDetails;
    use super::AuthorizationDetailsError;
    use super::CredentialConfigEntry;
    use super::CredentialEntry;
    use super::EntryContainer;
    use crate::metadata::issuer_metadata::CredentialConfigurationId;

    #[test]
    fn test_authorization_details_from_credential_ids_and_identifiers() {
        let credential_config_id_a = CredentialConfigurationId::from("credential_identifer_a".to_string());
        let credential_config_id_b = CredentialConfigurationId::from("credential_identifer_b".to_string());
        let credential_ids_and_identifiers = vec_nonempty![
            (&credential_config_id_b, "id_1_b".to_string()),
            (&credential_config_id_a, "id_1_a".to_string()),
            (&credential_config_id_a, "id_2_a".to_string()),
            (&credential_config_id_a, "id_3_a".to_string()),
            (&credential_config_id_b, "id_2_b".to_string()),
            (&credential_config_id_b, "id_2_b".to_string())
        ];

        let authorization_details =
            AuthorizationDetails::from_credential_ids_and_identifiers(credential_ids_and_identifiers);

        let entries = VecNonEmpty::from(authorization_details)
            .into_iter()
            .map(|entry_container| {
                (
                    entry_container.entry.config_entry.credential_configuration_id,
                    entry_container.entry.credential_identifiers,
                )
            })
            .sorted_by(|(left, _), (right, _)| Ord::cmp(left.as_ref(), right.as_ref()))
            .collect_vec();

        assert_eq!(
            entries,
            vec![
                (
                    credential_config_id_a,
                    vec_nonempty!["id_1_a".to_string(), "id_2_a".to_string(), "id_3_a".to_string()].into()
                ),
                (
                    credential_config_id_b,
                    vec_nonempty!["id_1_b".to_string(), "id_2_b".to_string()].into()
                )
            ]
        );
    }

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
        // Deserializing a completely different type of `authorization_details` entry should return an error.
        let _ = serde_json::from_value::<EntryContainer<CredentialConfigEntry>>(other_type_auth_details_example_json())
            .expect_err("deserializing EntryContainer should fail");
    }

    #[test]
    fn test_authorization_details_deserialize_incorrect_type() {
        // Deserializing an `authorization_details` entry where only the type field is incorrect should return an error.
        let mut json = auth_request_auth_details_example_json();
        *json.get_mut("type").unwrap() = serde_json::Value::String("account_information".to_string());

        let _ = serde_json::from_value::<EntryContainer<CredentialConfigEntry>>(json)
            .expect_err("deserializing EntryContainer should fail");
    }

    #[test]
    fn test_wallet_authorization_details_deserialize_and_serialize() {
        let entry_container =
            serde_json::from_value::<EntryContainer<CredentialConfigEntry>>(auth_request_auth_details_example_json())
                .expect("deserializing EntryContainer should succeed");

        assert_eq!(
            entry_container.entry.credential_configuration_id.as_ref(),
            "UniversityDegreeCredential"
        );

        let serialized = serde_json::to_value(entry_container).expect("serializing EntryContainer should succeed");

        assert_eq!(serialized, auth_request_auth_details_example_json());

        // Deserializing as `CredentialEntry` should fail, as it requires `credential_identifiers`.
        let _ = serde_json::from_value::<EntryContainer<CredentialEntry>>(auth_request_auth_details_example_json())
            .expect_err("deserializing EntryContainer should fail");
    }

    #[test]
    fn test_issuer_authorization_details_deserialize_and_serialize() {
        let entry_container =
            serde_json::from_value::<EntryContainer<CredentialEntry>>(token_response_auth_details_example_json())
                .expect("deserializing EntryContainer should succeed");

        assert_eq!(
            entry_container.entry.config_entry.credential_configuration_id.as_ref(),
            "UniversityDegreeCredential"
        );
        assert_eq!(
            entry_container
                .entry
                .credential_identifiers
                .iter()
                .map(String::as_str)
                .collect_vec(),
            vec!["CivilEngineeringDegree-2023", "ElectricalEngineeringDegree-2023"]
        );

        let serialized = serde_json::to_value(entry_container).expect("serializing EntryContainer should succeed");

        assert_eq!(serialized, token_response_auth_details_example_json());
    }

    #[test]
    fn test_authorization_details_with_claims_deserialize() {
        let auth_details =
            serde_json::from_value::<EntryContainer<CredentialConfigEntry>>(auth_details_example_with_claims_json())
                .expect("deserializing EntryContainer should succeed");

        assert_eq!(
            auth_details.entry.credential_configuration_id.as_ref(),
            "org.iso.18013.5.1.mDL"
        );

        // Note that we cannot test re-serialization, as we do not support the `claims` field.
    }

    #[test]
    fn test_authorization_details_deserialize_ok() {
        let json = json!([
            auth_request_auth_details_example_json(),
            auth_details_example_with_claims_json()
        ]);

        let entries = serde_json::from_value::<VecNonEmpty<EntryContainer<CredentialConfigEntry>>>(json)
            .expect("deserializing Vec of EntryContainer should succeed");

        let auth_details =
            AuthorizationDetails::try_new(entries).expect("creating AuthorizationDetails should succeed");

        assert_eq!(auth_details.as_ref().len().get(), 2);
    }

    #[test]
    fn test_authorization_details_deserialize_error_duplicate_credential_config_ids() {
        let json = json!([
            auth_request_auth_details_example_json(),
            auth_details_example_with_claims_json(),
            auth_request_auth_details_example_json()
        ]);

        let entries = serde_json::from_value::<VecNonEmpty<EntryContainer<CredentialConfigEntry>>>(json)
            .expect("deserializing Vec of EntryContainer should succeed");

        let error = AuthorizationDetails::try_new(entries).expect_err("creating AuthorizationDetails should fail");

        assert_matches!(
            error,
            AuthorizationDetailsError::DuplicateCredentialConfigIds(duplicate_ids)
                if duplicate_ids == vec!["UniversityDegreeCredential".to_string().into()]
        );
    }
}
