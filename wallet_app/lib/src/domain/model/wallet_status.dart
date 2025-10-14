sealed class WalletStatus {}

class WalletStatusReady extends WalletStatus {}

class WalletStatusTransferring extends WalletStatus {
  final bool canRetry;

  WalletStatusTransferring({this.canRetry = false});
}
