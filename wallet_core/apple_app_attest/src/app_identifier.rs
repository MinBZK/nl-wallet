use derive_more::derive::AsRef;
use derive_more::derive::Display;
use sha2::Digest;
use sha2::Sha256;

#[derive(Debug, Clone, AsRef, Display)]
#[display("{identifier}")]
pub struct AppIdentifier {
    #[as_ref(str)]
    identifier: String,
    bundle_identifier_offset: usize,
    hash: [u8; 32],
}

impl AppIdentifier {
    pub fn new(prefix: impl AsRef<str>, bundle_identifier: impl AsRef<str>) -> Self {
        let prefix = prefix.as_ref();

        let identifier = format!("{}.{}", prefix, bundle_identifier.as_ref());
        let bundle_identifier_offset = prefix.len() + 1;
        // Eagerly calculate the SHA256 hash, as it may be used multiple times.
        let hash = Sha256::digest(&identifier).into();

        Self {
            identifier,
            bundle_identifier_offset,
            hash,
        }
    }

    pub fn prefix(&self) -> &str {
        &self.identifier[..self.bundle_identifier_offset - 1]
    }

    pub fn bundle_identifier(&self) -> &str {
        &self.identifier[self.bundle_identifier_offset..]
    }

    pub fn sha256_hash(&self) -> &[u8] {
        &self.hash
    }
}

#[cfg(feature = "xcode_env")]
impl Default for AppIdentifier {
    fn default() -> Self {
        // When this crate is compiled as part of an Xcode build
        // chain the environment variables below should be set.
        let (Some(team_id), Some(bundle_id)) = (
            option_env!("DEVELOPMENT_TEAM"),
            option_env!("PRODUCT_BUNDLE_IDENTIFIER"),
        ) else {
            panic!("Xcode environment variables are not defined")
        };

        Self::new(team_id, bundle_id)
    }
}

#[cfg(feature = "mock")]
mod mock {
    use super::AppIdentifier;

    impl AppIdentifier {
        pub fn new_mock() -> Self {
            AppIdentifier::new("1234567890", "com.example.app")
        }
    }
}
