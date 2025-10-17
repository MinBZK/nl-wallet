sealed class WalletState {}

class WalletStateReady extends WalletState {}

class WalletStateTransferPossible extends WalletState {}

class WalletStateTransferring extends WalletState {
  final TransferRole role;

  WalletStateTransferring(this.role);
}

enum TransferRole { source, target }
