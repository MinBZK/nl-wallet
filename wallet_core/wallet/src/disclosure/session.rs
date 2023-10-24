use url::Url;

use nl_wallet_mdoc::{utils::serialization::cbor_deserialize, ReaderEngagement};

use super::{DisclosureSession, DisclosureSessionError};

// TODO: Implement actual disclosure.
#[allow(dead_code)]
pub struct HttpDisclosureSession {
    reader_engagement: ReaderEngagement,
    return_url: Option<Url>,
}

impl DisclosureSession for HttpDisclosureSession {
    fn start(reader_engagement_bytes: &[u8], return_url: Option<Url>) -> Result<Self, DisclosureSessionError> {
        let reader_engagement = cbor_deserialize(reader_engagement_bytes)?;

        let session = HttpDisclosureSession {
            reader_engagement,
            return_url,
        };

        Ok(session)
    }
}
