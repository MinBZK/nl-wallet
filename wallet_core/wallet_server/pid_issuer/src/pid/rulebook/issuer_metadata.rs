use attestation_data::attributes::Attribute;

use crate::pid::rulebook::constants::PID_ISSUING_AUTHORITY;
use crate::pid::rulebook::constants::PID_ISSUING_COUNTRY;

#[derive(Clone)]
pub struct PidIssuerMetadata {
    pub issuing_country: String,
    pub issuing_authority: String,
}

impl PidIssuerMetadata {
    pub(crate) fn into_attributes(self) -> [(String, Attribute); 2] {
        [
            (PID_ISSUING_COUNTRY.to_owned(), Attribute::Text(self.issuing_country)),
            (
                PID_ISSUING_AUTHORITY.to_owned(),
                Attribute::Text(self.issuing_authority),
            ),
        ]
    }
}
