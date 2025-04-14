use derive_more::AsRef;
use derive_more::From;
use serde::Deserialize;
use serde::Serialize;

/// Communicates that a type is optional in the specification it is derived from but implemented as mandatory due to
/// various reasons.
#[derive(Debug, Clone, From, AsRef, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecOptional<T>(T);

impl<T> SpecOptional<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}
