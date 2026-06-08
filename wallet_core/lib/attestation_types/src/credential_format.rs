use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, strum::Display, SerializeDisplay, DeserializeFromStr)]
#[strum(serialize_all = "snake_case")]
pub enum Format {
    MsoMdoc,
    #[strum(serialize = "dc+sd-jwt")]
    SdJwt,
}
