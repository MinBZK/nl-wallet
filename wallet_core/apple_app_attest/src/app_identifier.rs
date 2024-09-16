use std::fmt::{self, Display, Formatter};

use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct AppIdentifier {
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

impl Display for AppIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.identifier.fmt(f)
    }
}

impl AsRef<str> for AppIdentifier {
    fn as_ref(&self) -> &str {
        &self.identifier
    }
}
