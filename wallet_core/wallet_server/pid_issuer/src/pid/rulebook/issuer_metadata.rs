use attestation_data::attributes::Attribute;

use crate::pid::rulebook::constants::PID_ISSUING_AUTHORITY;
use crate::pid::rulebook::constants::PID_ISSUING_COUNTRY;
use crate::pid::rulebook::country_code::IsoCountryCode;
use crate::pid::rulebook::country_code::RulebookCountryCode;

#[derive(Clone)]
pub struct PidIssuerMetadata {
    pub issuing_country: IsoCountryCode,
    pub issuing_authority: String,
}

impl PidIssuerMetadata {
    pub(crate) fn into_attributes(self) -> [(String, Attribute); 2] {
        [
            (
                PID_ISSUING_COUNTRY.to_owned(),
                RulebookCountryCode::Iso(self.issuing_country).into(),
            ),
            (
                PID_ISSUING_AUTHORITY.to_owned(),
                Attribute::Text(self.issuing_authority),
            ),
        ]
    }
}
