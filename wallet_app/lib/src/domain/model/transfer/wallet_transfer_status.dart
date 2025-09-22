enum WalletTransferStatus {
  waitingForScan, // only used on source device
  waitingForApproval,
  transferring,
  cancelled,
  error,
  success,
}
