use http_utils::urls;
use jwe::encryption::JwePublicKey;
use jwk_simple::jwk::Key;
use serde::Deserialize;
use serde::Serialize;
use serde_with::TryFromInto;
use serde_with::json::JsonString;
use serde_with::serde_as;
use url::Url;

use crate::config::UNIVERSAL_LINK_BASE_URL;
use crate::transfer::TransferSessionId;

#[derive(Debug, thiserror::Error)]
pub enum TransferUriError {
    #[error("invalid transfer uri: {0}")]
    InvalidUri(String),

    #[error("error deserializing query parameters: {0}")]
    QueryDeserialization(#[from] serde_urlencoded::de::Error),

    #[error("error serializing query parameters: {0}")]
    QuerySerialization(#[from] serde_urlencoded::ser::Error),

    #[error("error decoding from base64: {0}")]
    Base64Decoding(#[from] base64::DecodeError),
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferQuery {
    #[serde(rename = "s")]
    pub session_id: TransferSessionId,

    #[serde(rename = "k")]
    #[serde_as(as = "JsonString<TryFromInto<Key>>")]
    pub public_key: JwePublicKey,
}

impl TryFrom<Url> for TransferQuery {
    type Error = TransferUriError;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        let Some(query) = value.fragment() else {
            return Err(TransferUriError::InvalidUri(value.to_string()));
        };

        let query: TransferQuery = serde_urlencoded::from_str(query)?;

        Ok(query)
    }
}

impl TryFrom<TransferQuery> for Url {
    type Error = TransferUriError;

    fn try_from(value: TransferQuery) -> Result<Self, Self::Error> {
        let mut url: Url = urls::transfer_base_uri(&UNIVERSAL_LINK_BASE_URL).into_inner();
        url.set_fragment(Some(serde_urlencoded::to_string(value)?.as_str()));

        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use jwe::algorithm::EcdhAlgorithm;
    use jwe::decryption::JweEcdhSecretKey;
    use url::Host;
    use url::Url;
    use uuid::Uuid;

    use crate::transfer::uri::TransferQuery;

    #[test]
    fn test_transfer_query() {
        let transfer_session_id = Uuid::new_v4();
        let secret_key = JweEcdhSecretKey::new_random(None, EcdhAlgorithm::EcdhEsA256kw);

        let transfer_query = TransferQuery {
            session_id: transfer_session_id.into(),
            public_key: secret_key.to_jwe_public_key(),
        };
        let url: Url = transfer_query.try_into().unwrap();

        assert_eq!(url.scheme(), "walletdebuginteraction");
        assert_eq!(
            url.host().map(|h| h.to_owned()),
            Some(Host::parse("wallet.edi.rijksoverheid.nl").unwrap())
        );
        assert_eq!(url.path(), "/transfer");
        assert_eq!(url.query(), None);
        assert!(url.fragment().is_some());

        let query: TransferQuery = serde_urlencoded::from_str(url.fragment().unwrap()).unwrap();
        assert_eq!(query.session_id, transfer_session_id.into());
        assert_eq!(query.public_key.key(), secret_key.key().public_key());
    }
}
