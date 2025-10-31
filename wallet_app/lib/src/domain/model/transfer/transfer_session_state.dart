enum TransferSessionState {
  /// The initial state; waiting for the user to scan the QR code on the source device.
  ///
  /// This state is not relevant for the source device.
  created,

  /// Waiting for the user to confirm the transfer on the source device.
  paired,

  /// The user successfully confirmed (with pin), and the wallet data is ready to be uploaded by the source device.
  confirmed,

  /// The wallet data is ready to be downloaded on the target device.
  uploaded,

  /// The transfer completed successfully.
  success,

  /// The transfer has been cancelled by the user on either device.
  cancelled,

  /// An error occurred during the transfer.
  error,
}
