sealed class WalletStatus {}

class WalletStatusReady extends WalletStatus {}

class WalletStatusTransferPossible extends WalletStatus {}

class WalletStatusTransferring extends WalletStatus {
  final TransferRole role;

  WalletStatusTransferring(this.role);
}

enum TransferRole { source, target }
