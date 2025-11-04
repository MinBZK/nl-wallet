part of 'wallet_transfer_target_bloc.dart';

const _kTransferSteps = 4;

sealed class WalletTransferTargetState extends Equatable {
  bool get canGoBack => false;

  bool get didGoBack => false;

  FlowProgress? get stepperProgress => null;

  const WalletTransferTargetState();

  @override
  List<Object?> get props => [canGoBack, didGoBack, stepperProgress];
}

/// Represents the state where introductory information about the wallet transfer
/// is displayed to the user. This is also the initial state.
class WalletTransferIntroduction extends WalletTransferTargetState {
  @override
  final bool didGoBack;

  const WalletTransferIntroduction({this.didGoBack = false});
}

/// Represents the state where the contents of the QR code are being generated
class WalletTransferLoadingQrData extends WalletTransferTargetState {
  const WalletTransferLoadingQrData();
}

/// Represents the state where the QR code is presented to the user.
class WalletTransferAwaitingQrScan extends WalletTransferTargetState {
  @override
  bool get canGoBack => true;

  @override
  FlowProgress? get stepperProgress => const FlowProgress(currentStep: 1, totalSteps: _kTransferSteps);

  final String qrContents;

  const WalletTransferAwaitingQrScan(this.qrContents);

  @override
  List<Object?> get props => [...super.props, qrContents];
}

/// Represents the state where the user needs to confirm their PIN on the source device
/// to begin the transfer process.
class WalletTransferAwaitingConfirmation extends WalletTransferTargetState {
  @override
  FlowProgress? get stepperProgress => const FlowProgress(currentStep: 2, totalSteps: _kTransferSteps);

  const WalletTransferAwaitingConfirmation();
}

/// Represents the state where the transfer is actively in progress.
class WalletTransferTransferring extends WalletTransferTargetState {
  final bool isReceiving;

  @override
  FlowProgress? get stepperProgress => const FlowProgress(currentStep: 3, totalSteps: _kTransferSteps);

  const WalletTransferTransferring({required this.isReceiving});

  @override
  List<Object?> get props => [isReceiving, super.props];
}

/// Represents the state when the wallet transfer has been successfully completed.
class WalletTransferSuccess extends WalletTransferTargetState {
  @override
  FlowProgress? get stepperProgress => const FlowProgress(currentStep: _kTransferSteps, totalSteps: _kTransferSteps);

  const WalletTransferSuccess();
}

/// Represents the state when the wallet transfer was stopped by the user.
class WalletTransferStopped extends WalletTransferTargetState {
  const WalletTransferStopped();
}

/// Represents the UI state for generic/unknown errors
class WalletTransferGenericError extends WalletTransferTargetState implements ErrorState {
  @override
  final ApplicationError error;

  const WalletTransferGenericError(this.error);

  @override
  List<Object?> get props => [...super.props, error];
}

/// Represents the UI state for network/internet related errors
class WalletTransferNetworkError extends WalletTransferTargetState implements NetworkErrorState {
  @override
  final NetworkError error;

  @override
  bool get hasInternet => error.hasInternet;

  @override
  int? get statusCode => error.statusCode;

  const WalletTransferNetworkError(this.error);

  @override
  List<Object?> get props => [...super.props, error];
}

/// Represents the UI state for session errors
class WalletTransferSessionExpired extends WalletTransferTargetState implements ErrorState {
  @override
  final ApplicationError error;

  const WalletTransferSessionExpired(this.error);

  @override
  List<Object?> get props => [...super.props, error];
}

/// Represents the UI state for transfer errors (i.e. errors that occur after the transfer was initiated)
class WalletTransferFailed extends WalletTransferTargetState implements ErrorState {
  @override
  final ApplicationError error;

  const WalletTransferFailed(this.error);

  @override
  List<Object?> get props => [...super.props, error];
}
