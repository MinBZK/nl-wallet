use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuanceValidity {
    pub signed: DateTime<Utc>,
    #[serde(flatten)]
    pub validity_window: ValidityWindow,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidityWindow {
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
}

impl IssuanceValidity {
    pub fn new(signed: DateTime<Utc>, valid_from: Option<DateTime<Utc>>, valid_until: Option<DateTime<Utc>>) -> Self {
        Self {
            signed,
            validity_window: ValidityWindow {
                valid_from,
                valid_until,
            },
        }
    }
}

impl TryFrom<&mdoc::iso::ValidityInfo> for IssuanceValidity {
    type Error = chrono::ParseError;

    fn try_from(value: &mdoc::iso::ValidityInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            signed: (&value.signed).try_into()?,
            validity_window: ValidityWindow {
                valid_from: Some((&value.valid_from).try_into()?),
                valid_until: Some((&value.valid_until).try_into()?),
            },
        })
    }
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use std::ops::Add;

    use chrono::DateTime;
    use chrono::Months;

    use crate::validity::IssuanceValidity;
    use crate::validity::ValidityWindow;

    impl IssuanceValidity {
        pub fn new_valid_mock() -> Self {
            Self {
                signed: DateTime::UNIX_EPOCH,
                validity_window: ValidityWindow::new_valid_mock(),
            }
        }
    }

    impl ValidityWindow {
        pub fn new_valid_mock() -> Self {
            Self {
                valid_from: Some(DateTime::UNIX_EPOCH),
                valid_until: Some(DateTime::UNIX_EPOCH.add(Months::new(12000))),
            }
        }
    }
}
