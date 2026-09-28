use crypto::utils::random_string;
use nutype::nutype;

#[nutype(derive(Debug, Clone, TryFrom, AsRef, Into), validate(regex = r"^[A-Za-z0-9]+$"))]
pub struct ExternalId(String);

impl ExternalId {
    /// Length of the external id for status lists used in the url (alphanumeric characters)
    const SIZE: usize = 12;

    pub fn generate() -> Self {
        Self::try_from(random_string(Self::SIZE))
            .expect("random_string should only generate ASCII alphanumeric characters")
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
        assert_eq!(result.expect_err("should fail"), ExternalIdError::RegexViolated);
    }
}
