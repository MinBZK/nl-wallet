use rand::Rng;

use crate::{digid, settings::MockAttributes};

use crate::app::BsnLookup;

pub struct MockBsnLookup(Vec<String>);

impl Default for MockBsnLookup {
    fn default() -> Self {
        Self(vec!["999991772".to_owned()])
    }
}

impl From<Vec<MockAttributes>> for MockBsnLookup {
    fn from(value: Vec<MockAttributes>) -> Self {
        Self(value.iter().map(|p| p.person.bsn.clone()).collect())
    }
}

impl BsnLookup for MockBsnLookup {
    async fn bsn(&self, _access_token: &str) -> Result<String, digid::Error> {
        Ok(self.0[rand::thread_rng().gen_range(0..self.0.len())].clone())
    }
}
