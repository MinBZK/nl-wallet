use std::fmt::{self, Display, Formatter};

pub struct AppIdentifier {
    identifier: String,
    bundle_identifier_offset: usize,
}

impl AppIdentifier {
    pub fn new(prefix: impl AsRef<str>, bundle_identifier: impl AsRef<str>) -> Self {
        let prefix = prefix.as_ref();

        Self {
            identifier: format!("{}.{}", prefix, bundle_identifier.as_ref()),
            bundle_identifier_offset: prefix.len() + 1,
        }
    }

    pub fn prefix(&self) -> &str {
        &self.identifier[..self.bundle_identifier_offset - 1]
    }

    pub fn bundle_identifier(&self) -> &str {
        &self.identifier[self.bundle_identifier_offset..]
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
