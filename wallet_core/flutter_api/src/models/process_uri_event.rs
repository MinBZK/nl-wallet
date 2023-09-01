pub enum ProcessUriEvent {
    PidIssuance(PidIssuanceEvent),
}

pub enum PidIssuanceEvent {
    Authenticating,
    Success,
    Error,
}
