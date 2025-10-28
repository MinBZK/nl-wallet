part of 'wallet_transfer_source_bloc.dart';

abstract class WalletTransferSourceEvent extends Equatable {
  const WalletTransferSourceEvent();

  @override
  List<Object?> get props => [];
}

/// Event triggered when the wallet transfer process is acknowledged.
///
/// This event contains the URI necessary to acknowledge the transfer.
class WalletTransferAcknowledgeTransferEvent extends WalletTransferSourceEvent {
  /// The URI or identifier needed to acknowledge the transfer.
  final String uri;

  const WalletTransferAcknowledgeTransferEvent(this.uri);

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

/// Event used internally by the [WalletTransferSourceBloc] to update the state
class WalletTransferUpdateStateEvent extends WalletTransferSourceEvent {
  final WalletTransferSourceState state;

  const WalletTransferUpdateStateEvent(this.state);

  @override
  List<Object?> get props => [state];
}
