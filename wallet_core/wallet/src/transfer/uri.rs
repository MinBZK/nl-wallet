use std::collections::HashMap;

use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use url::Url;
use uuid::Uuid;

use http_utils::urls;

use crate::config::UNIVERSAL_LINK_BASE_URL;
use crate::transfer::TransferSessionId;

const TRANSFER_SESSION_ID_QUERY_PARAM_KEY: &str = "s";
const KEY_QUERY_PARAM_KEY: &str = "k";

#[derive(Debug, thiserror::Error)]
pub enum TransferUriError {
    #[error("invalid transfer uri: {0}")]
    InvalidUri(String),

    #[error("missing query parameter: {0}, in: {1}")]
    MissingQueryParameter(String, String),

    #[error("error deserializing query parameters: {0}")]
    QueryParameterDeserialization(#[from] serde_urlencoded::de::Error),

    #[error("error serializing query parameters: {0}")]
    QueryParameterSerialization(#[from] serde_urlencoded::ser::Error),

    #[error("invalid uuid: {0}")]
    InvalidUuid(#[from] uuid::Error),

    #[error("error decoding from base64: {0}")]
    Base64Decoding(#[from] base64::DecodeError),
}

#[derive(Debug, PartialEq)]
pub struct TransferUri {
    pub transfer_session_id: TransferSessionId,
    pub key: Vec<u8>,
}

impl TryFrom<Url> for TransferUri {
    type Error = TransferUriError;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        let Some(query) = value.fragment() else {
            return Err(TransferUriError::InvalidUri(value.to_string()));
        };

        let mut query_params: HashMap<String, String> = serde_urlencoded::from_str(query)?;

        let transfer_session_id = Uuid::from_slice(
            &BASE64_URL_SAFE_NO_PAD.decode(
                query_params
                    .remove(TRANSFER_SESSION_ID_QUERY_PARAM_KEY)
                    .ok_or(TransferUriError::MissingQueryParameter(
                        String::from(TRANSFER_SESSION_ID_QUERY_PARAM_KEY),
                        value.to_string(),
                    ))?
                    .as_bytes(),
            )?,
        )?;

        let key = BASE64_URL_SAFE_NO_PAD.decode(
            query_params
                .remove(KEY_QUERY_PARAM_KEY)
                .ok_or(TransferUriError::MissingQueryParameter(
                    String::from(KEY_QUERY_PARAM_KEY),
                    value.to_string(),
                ))?
                .as_bytes(),
        )?;

        Ok(TransferUri {
            transfer_session_id: transfer_session_id.into(),
            key,
        })
    }
}

impl TryFrom<TransferUri> for Url {
    type Error = TransferUriError;

    fn try_from(value: TransferUri) -> Result<Self, Self::Error> {
        let query = serde_urlencoded::to_string(&[
            (
                TRANSFER_SESSION_ID_QUERY_PARAM_KEY,
                BASE64_URL_SAFE_NO_PAD.encode(value.transfer_session_id.as_ref()),
            ),
            (KEY_QUERY_PARAM_KEY, BASE64_URL_SAFE_NO_PAD.encode(value.key.as_slice())),
        ])?;

        let mut url: Url = urls::transfer_base_uri(&UNIVERSAL_LINK_BASE_URL).into_inner();
        url.set_fragment(Some(query.as_str()));

        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;
    use url::Host;
    use url::Url;
    use url::form_urlencoded;
    use uuid::Uuid;

    use crypto::utils::random_bytes;

    use crate::transfer::uri::TransferUri;

    #[test]
    fn test_transfer_uri() {
        let transfer_session_id = Uuid::new_v4();
        let transfer_key = random_bytes(32);

        let transfer_uri = TransferUri {
            transfer_session_id: transfer_session_id.into(),
            key: transfer_key.clone(),
        };
        let url: Url = transfer_uri.try_into().unwrap();

        assert_eq!(url.scheme(), "walletdebuginteraction");
        assert_eq!(
            url.host().map(|h| h.to_owned()),
            Some(Host::parse("wallet.edi.rijksoverheid.nl").unwrap())
        );
        assert_eq!(url.path(), "/transfer");
        assert_eq!(url.query(), None);
        assert!(url.fragment().is_some());

        let mut pairs = form_urlencoded::parse(url.fragment().unwrap().as_bytes());

        let (key, value) = pairs.next().unwrap();
        assert_eq!(key, "s");
        assert_eq!(
            BASE64_URL_SAFE_NO_PAD
                .decode(value.as_ref())
                .map(|id| Uuid::from_slice(id.as_ref())),
            Ok(Ok(transfer_session_id))
        );

        let (key, value) = pairs.next().unwrap();
        assert_eq!(key, "k");
        assert_eq!(BASE64_URL_SAFE_NO_PAD.decode(value.as_ref()).unwrap().len(), 32);

        assert!(pairs.next().is_none());

        let from_url = TransferUri::try_from(url).unwrap();
        assert_eq!(
            from_url,
            TransferUri {
                transfer_session_id: transfer_session_id.into(),
                key: transfer_key,
            }
        );
    }
}
