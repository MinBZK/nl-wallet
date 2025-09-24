part of 'wallet_transfer_source_bloc.dart';

const _kTransferSteps = 4;

sealed class WalletTransferSourceState extends Equatable {
  bool get canGoBack => false;

  bool get didGoBack => false;

  FlowProgress? get stepperProgress => null;

  const WalletTransferSourceState();

  @override
  List<Object?> get props => [canGoBack, didGoBack, stepperProgress];
}

/// Represents the initial state of the wallet transfer
class WalletTransferInitial extends WalletTransferSourceState {
  const WalletTransferInitial();
}

/// Represents the loading state, where the qr containing the tranfer request is
/// being processed.
class WalletTransferLoading extends WalletTransferSourceState {
  const WalletTransferLoading();
}

/// Represents the state where introductory information about the wallet transfer
/// is displayed to the user.
class WalletTransferIntroduction extends WalletTransferSourceState {
  @override
  final bool didGoBack;

  @override
  FlowProgress? get stepperProgress => const FlowProgress(currentStep: 1, totalSteps: _kTransferSteps);

  const WalletTransferIntroduction({this.didGoBack = false});
}

/// Represents the state where the user needs to confirm their PIN to proceed
/// with the transfer.
class WalletTransferConfirmPin extends WalletTransferSourceState {
  @override
  FlowProgress? get stepperProgress => const FlowProgress(currentStep: 2, totalSteps: _kTransferSteps);

  @override
  bool get canGoBack => true;

  const WalletTransferConfirmPin();
}

/// Represents the state where the transfer is actively in progress.
class WalletTransferTransferring extends WalletTransferSourceState {
  @override
  FlowProgress? get stepperProgress => const FlowProgress(currentStep: 3, totalSteps: _kTransferSteps);

  const WalletTransferTransferring();
}

/// Represents the state when the wallet transfer has been successfully completed.
class WalletTransferSuccess extends WalletTransferSourceState {
  @override
  FlowProgress? get stepperProgress => const FlowProgress(currentStep: _kTransferSteps, totalSteps: _kTransferSteps);

  const WalletTransferSuccess();
}

/// Represents the state when the wallet transfer was stopped by the user.
class WalletTransferStopped extends WalletTransferSourceState {
  const WalletTransferStopped();
}

/// Represents the UI state for generic/unknown errors
class WalletTransferGenericError extends WalletTransferSourceState implements ErrorState {
  @override
  final ApplicationError error;

  const WalletTransferGenericError(this.error);

  @override
  List<Object?> get props => [...super.props, error];
}

/// Represents the UI state for network/internet related errors
class WalletTransferNetworkError extends WalletTransferSourceState implements NetworkErrorState {
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
class WalletTransferSessionExpired extends WalletTransferSourceState implements ErrorState {
  @override
  final ApplicationError error;

  const WalletTransferSessionExpired(this.error);

  @override
  List<Object?> get props => [...super.props, error];
}

/// Represents the UI state for transfer errors (i.e. errors that occur after the transfer was initiated)
class WalletTransferFailed extends WalletTransferSourceState implements ErrorState {
  @override
  final ApplicationError error;

  const WalletTransferFailed(this.error);

  @override
  List<Object?> get props => [...super.props, error];
}
