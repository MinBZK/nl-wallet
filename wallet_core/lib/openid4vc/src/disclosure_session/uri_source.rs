use crate::verifier::SessionType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
// Symmetrical to `SessionType`.
#[strum(serialize_all = "snake_case")]
pub enum DisclosureUriSource {
    Link,
    QrCode,
}

impl DisclosureUriSource {
    pub fn new(is_qr_code: bool) -> Self {
        if is_qr_code {
            Self::QrCode
        } else {
            Self::Link
        }
    }

    /// Returns the expected session type for a source of the received [`ReaderEngagement`].
    pub fn session_type(&self) -> SessionType {
        match self {
            Self::Link => SessionType::SameDevice,
            Self::QrCode => SessionType::CrossDevice,
        }
    }
}
