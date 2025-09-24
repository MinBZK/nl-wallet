part of 'wallet_transfer_target_bloc.dart';

abstract class WalletTransferTargetEvent extends Equatable {
  const WalletTransferTargetEvent();

  @override
  List<Object?> get props => [];
}

/// Event triggered when the user chooses to initiate the wallet transfer
class WalletTransferOptInEvent extends WalletTransferTargetEvent {
  const WalletTransferOptInEvent();
}

/// Event triggered when the user chooses to restart the wallet transfer
class WalletTransferRestartEvent extends WalletTransferTargetEvent {
  const WalletTransferRestartEvent();
}

/// Event triggered when the user chooses to skip the wallet transfer
class WalletTransferOptOutEvent extends WalletTransferTargetEvent {
  const WalletTransferOptOutEvent();
}

/// Event triggered when the user requests to stop or cancel the ongoing transfer process.
class WalletTransferStopRequestedEvent extends WalletTransferTargetEvent {
  const WalletTransferStopRequestedEvent();
}

/// Event triggered when the user requests to go back.
class WalletTransferBackPressedEvent extends WalletTransferTargetEvent {
  const WalletTransferBackPressedEvent();
}
