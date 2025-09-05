pub enum WalletTransferState {
    WaitingForScan,
    WaitingForApproval,
    Transferring,
    Error,
    Success,
}
