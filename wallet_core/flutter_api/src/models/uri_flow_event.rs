pub enum UriFlowEvent {
    DigidAuth { state: DigidState },
}

pub enum DigidState {
    Authenticating,
    Success,
    Error,
}
