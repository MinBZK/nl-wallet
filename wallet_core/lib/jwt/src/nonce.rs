use derive_more::AsRef;
use derive_more::Display;
use derive_more::From;
use derive_more::Into;
use serde::Deserialize;
use serde::Serialize;

use crypto::utils::random_string;

#[derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, From, Into, Display, Serialize, Deserialize)]
#[as_ref(str)]
pub struct Nonce(String);

impl Nonce {
    pub fn new_random() -> Self {
        Self(random_string(32))
    }
}
