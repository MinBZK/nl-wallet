use attestation_data::attributes::Attribute;

/// The [ISO/IEC 5218] `sex` codes.
///
/// [ISO/IEC 5218]: https://en.wikipedia.org/wiki/ISO/IEC_5218
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsoSex {
    NotKnown = 0,
    Male = 1,
    Female = 2,
    NotApplicable = 9,
}

/// PID Rulebook (section 2.3) `sex` attribute values.
///
/// [`Self::Iso`] covers the [ISO/IEC 5218] values (see [`IsoSex`]); the remaining variants are Rulebook additions on
/// top of that standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulebookSex {
    Iso(IsoSex),
    Other,
    Inter,
    Diverse,
    Open,
}

impl From<RulebookSex> for Attribute {
    fn from(value: RulebookSex) -> Attribute {
        match value {
            RulebookSex::Iso(iso) => Attribute::Number(iso as i64),
            RulebookSex::Other => Attribute::Number(3),
            RulebookSex::Inter => Attribute::Number(4),
            RulebookSex::Diverse => Attribute::Number(5),
            RulebookSex::Open => Attribute::Number(6),
        }
    }
}
