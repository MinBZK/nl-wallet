part of 'wallet_transfer_source_bloc.dart';

abstract class WalletTransferSourceEvent extends Equatable {
  const WalletTransferSourceEvent();

  @override
  List<Object?> get props => [];
}

/// Event triggered when the wallet transfer process is initiated.
///
/// This event contains the URI necessary to initiate the transfer.
class WalletTransferInitiateTransferEvent extends WalletTransferSourceEvent {
  /// The URI or identifier needed to initiate the transfer.
  final String uri;

  const WalletTransferInitiateTransferEvent(this.uri);

  @override
  List<Object?> get props => [...super.props, uri];
}

/// Event triggered when the user agrees to proceed with the transfer.
class WalletTransferAgreeEvent extends WalletTransferSourceEvent {
  const WalletTransferAgreeEvent();
}

/// Event triggered when the user has successfully confirmed their PIN.
class WalletTransferPinConfirmedEvent extends WalletTransferSourceEvent {
  const WalletTransferPinConfirmedEvent();
}

/// Event triggered when the user has successfully confirmed their PIN.
class WalletTransferPinConfirmationFailed extends WalletTransferSourceEvent {
  final ApplicationError error;

  const WalletTransferPinConfirmationFailed(this.error);

  @override
  List<Object?> get props => [...super.props, error];
}

/// Event triggered when the user requests to stop or cancel the ongoing transfer process.
class WalletTransferStopRequestedEvent extends WalletTransferSourceEvent {
  const WalletTransferStopRequestedEvent();
}

/// Event triggered when the user requests to go back.
class WalletTransferBackPressedEvent extends WalletTransferSourceEvent {
  const WalletTransferBackPressedEvent();
}
