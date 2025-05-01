use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

use base64::display::Base64Display;
use base64::engine::general_purpose::STANDARD;
use data_url::forgiving_base64::InvalidBase64;
use data_url::DataUrl;
use data_url::DataUrlError;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct DataUri {
    pub mime_type: String,
    pub data: Vec<u8>,
}

impl Display for DataUri {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "data:{};base64,{}",
            self.mime_type.as_str(),
            Base64Display::new(&self.data, &STANDARD)
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("data-uri error: {0}")]
    Uri(#[from] DataUrlError),
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] InvalidBase64),
}

impl FromStr for DataUri {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let url = DataUrl::process(s)?;
        Ok(DataUri {
            mime_type: url.mime_type().to_string(),
            data: url.decode_to_vec()?.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case("data:image/jpeg;base64,yv4=", DataUri {
        mime_type: "image/jpeg".to_string(),
        data: vec![0xca, 0xfe],
    })]
    #[case("data:image/svg+xml;utf8,<svg></svg>", DataUri {
        mime_type: "image/svg+xml".to_string(),
        data: b"<svg></svg>".to_vec(),
    })]
    #[test]
    fn parsing(#[case] value: &str, #[case] expected: DataUri) {
        assert_eq!(DataUri::from_str(value).unwrap(), expected);
    }

    #[rstest]
    #[case("https://example.com")]
    #[case("invalid")]
    #[test]
    fn parsing_error_url(#[case] value: &str) {
        assert_eq!(
            DataUri::from_str(value).unwrap_err().to_string(),
            "data-uri error: not a valid data url"
        );
    }

    #[test]
    fn parsing_error_decode() {
        assert_eq!(
            DataUri::from_str("data:image/jpeg;base64,/").unwrap_err().to_string(),
            "base64 decode error: lone alphabet symbol present"
        );
    }
}
