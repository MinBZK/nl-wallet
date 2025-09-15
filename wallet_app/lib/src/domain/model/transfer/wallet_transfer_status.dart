enum WalletTransferStatus {
  waitingForScan, // only used on source device
  waitingForApproval,
  transferring,
  error,
  success,
}
