pub struct ValidityWindow {
    pub valid_from: Option<String>,  // ISO8601
    pub valid_until: Option<String>, // ISO8601
}

impl From<wallet::attestation_data::ValidityWindow> for ValidityWindow {
    fn from(window: wallet::attestation_data::ValidityWindow) -> Self {
        ValidityWindow {
            valid_from: window.valid_from.map(|datetime| datetime.to_rfc3339()),
            valid_until: window.valid_until.map(|datetime| datetime.to_rfc3339()),
        }
    }
}
