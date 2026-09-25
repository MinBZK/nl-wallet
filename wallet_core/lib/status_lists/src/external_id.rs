use crypto::utils::random_string;
use derive_more::AsRef;
use derive_more::Into;

#[derive(Debug, Clone, AsRef, Into)]
pub struct ExternalId(String);

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq))]
#[error("invalid external id: {0}")]
pub struct ExternalIdError(String);

impl ExternalId {
    /// Length of the external id for status lists used in the url (alphanumeric characters)
    const SIZE: usize = 12;

    pub fn generate() -> Self {
        Self::try_from(random_string(Self::SIZE))
            .expect("random_string should only generate ASCII alphanumeric characters")
    }
}

impl TryFrom<String> for ExternalId {
    type Error = ExternalIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ExternalIdError(value));
        }
        if value.chars().all(|c| c.is_ascii_alphanumeric()) {
            Ok(Self(value))
        } else {
            Err(ExternalIdError(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::ExternalId;
    use super::ExternalIdError;

    #[rstest]
    #[case("abc")]
    #[case("abc123")]
    fn test_external_id_ok(#[case] value: &str) {
        let result = ExternalId::try_from(value.to_string());
        assert_eq!(result.expect("should succeed").as_ref(), value);
    }

    #[rstest]
    #[case("")]
    #[case(" abc ")]
    #[case("teßt")]
    #[case("..")]
    fn test_external_id_invalid(#[case] value: &str) {
        let result = ExternalId::try_from(value.to_string());
        assert_eq!(result.expect_err("should fail"), ExternalIdError(value.to_string()));
    }
}
