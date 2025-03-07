use serde::Deserialize;
use serde::Serialize;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(tag = "env", rename_all = "lowercase")]
pub enum DigidApp2AppConfiguration {
    Production { universal_link: Url },
    Preprod { host: String, universal_link: Url },
}

impl DigidApp2AppConfiguration {
    pub fn universal_link(&self) -> &Url {
        match self {
            Self::Production { universal_link } | Self::Preprod { universal_link, .. } => universal_link,
        }
    }

    pub fn host(&self) -> Option<&str> {
        match self {
            Self::Production { .. } => None,
            Self::Preprod { host, .. } => Some(host),
        }
    }
}
