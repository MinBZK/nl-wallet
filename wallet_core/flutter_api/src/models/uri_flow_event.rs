use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum UriFlowEvent {
    DigidAuth { state: DigidState },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DigidState {
    Authenticating,
    Success,
    Error,
}
