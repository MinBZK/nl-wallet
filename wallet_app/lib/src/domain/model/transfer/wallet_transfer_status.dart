enum WalletTransferStatus {
  /// The initial state; waiting for the user to scan the QR code on the source device.
  ///
  /// This state is not relevant for the source device.
  waitingForScan,

  /// Waiting for the user to approve the transfer on the source device.
  /// The state will progress to [transferring] once the source device has finished uploading the data.
  ///
  /// This state is not relevant for the source device.
  waitingForApprovalAndUpload,

  /// The wallet data is being transferred & processed.
  transferring,

  /// The transfer has been cancelled by the user on either device.
  cancelled,

  /// An error occurred during the transfer.
  error,

  /// The transfer completed successfully.
  success,
}
