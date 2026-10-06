use attestation_data::attributes::Attribute;

/// PID Rulebook (section 2.3) `sex` attribute values. Values 0, 1, 2 and 9 follow [ISO/IEC 5218]; 3-6 are Rulebook
/// additions on top of that standard, for gender identities not covered by ISO/IEC 5218.
///
/// [ISO/IEC 5218]: https://en.wikipedia.org/wiki/ISO/IEC_5218
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PidSex {
    NotKnown = 0,
    Male = 1,
    Female = 2,
    Other = 3,
    Inter = 4,
    Diverse = 5,
    Open = 6,
    NotApplicable = 9,
}

impl From<PidSex> for Attribute {
    fn from(value: PidSex) -> Attribute {
        Attribute::Number(value as i64)
    }
}
