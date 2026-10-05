use derive_more::AsRef;
use derive_more::From;
use derive_more::FromStr;
use derive_more::Into;
use http_utils::urls::HttpsUri;
use rcgen::SanType;
use rcgen::string::Ia5String;

#[derive(Debug, Clone, FromStr, AsRef, Into, From)]
pub struct SubjectAltNameUri(HttpsUri);

impl From<&SubjectAltNameUri> for SanType {
    fn from(san_uri: &SubjectAltNameUri) -> Self {
        let uri = Ia5String::try_from(san_uri.0.to_string()).expect("url is a valid uri");
        SanType::URI(uri)
    }
}
