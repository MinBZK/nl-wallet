use std::sync::LazyLock;

use attestation_data::attributes::Attributes;
use chrono::NaiveDate;
use utils::vec_nonempty;

use crate::pid::attributes::Address;
use crate::pid::attributes::Person;
use crate::pid::attributes::UncheckedBirthPlace;
use crate::pid::rulebook::country_code::IsoCountryCode;
use crate::pid::rulebook::issuer_metadata::PidIssuerMetadata;
use crate::pid::rulebook::sex::IsoSex;

pub static MOCK_PERSON: LazyLock<Person> = LazyLock::new(|| Person {
    bsn: "999991772".into(),
    family_name: "Jansen".into(),
    family_name_birth: Some("Jansen".into()),
    given_name: "Frouke".into(),
    birth_date: NaiveDate::from_ymd_opt(2000, 3, 24).unwrap(),
    birth_place: UncheckedBirthPlace {
        country: Some(IsoCountryCode::Netherlands.into()),
        region: None,
        locality: None,
    }
    .try_into()
    .unwrap(),
    nationalities: vec_nonempty![IsoCountryCode::Netherlands.into(), IsoCountryCode::Belgium.into()].into(),
    address: Some(Address {
        street: Some("Van Wijngaerdenstraat".into()),
        house_number: Some("1".into()),
        postal_code: Some("2596TW".into()),
        city: Some("Toetsoog".into()),
        state: None,
        country: Some(IsoCountryCode::Netherlands.into()),
    }),
    sex: Some(IsoSex::Female.into()),
});

const MOCK_RECOVERY_CODE: &str = "1234567";

static MOCK_ISSUER_METADATA: LazyLock<PidIssuerMetadata> = LazyLock::new(|| PidIssuerMetadata {
    issuing_country: IsoCountryCode::Netherlands,
    issuing_authority: "Issuing Authority".into(),
});

pub fn mock_pid_example() -> Attributes {
    MOCK_PERSON
        .clone()
        .into_sd_jwt_attributes(MOCK_RECOVERY_CODE.into(), MOCK_ISSUER_METADATA.clone())
}

pub fn mock_pid_mdoc_example() -> Attributes {
    MOCK_PERSON
        .clone()
        .into_mdoc_attributes(MOCK_RECOVERY_CODE.into(), MOCK_ISSUER_METADATA.clone())
}
