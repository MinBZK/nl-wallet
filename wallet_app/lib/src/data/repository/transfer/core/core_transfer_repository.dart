import 'package:wallet_core/core.dart';

import '../../../../domain/model/transfer/wallet_transfer_status.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../transfer_repository.dart';

class CoreTransferRepository implements TransferRepository {
  final TypedWalletCore _walletCore;

  CoreTransferRepository(this._walletCore);

  @override
  Future<String> initWalletTransfer() => _walletCore.initWalletTransfer();

  @override
  Future<void> pairWalletTransfer(String uri) => _walletCore.pairWalletTransfer(uri);

  @override
  Future<WalletInstructionResult> confirmWalletTransfer(String pin) => _walletCore.confirmWalletTransfer(pin);

  @override
  Future<void> transferWallet() => _walletCore.transferWallet();

  @override
  Future<void> receiveWalletTransfer() => _walletCore.receiveWalletTransfer();

  @override
  Future<void> cancelWalletTransfer() => _walletCore.cancelWalletTransfer();

  @override
  Future<WalletTransferStatus> getWalletTransferState() async {
    final result = await _walletCore.getWalletTransferState();
    return switch (result) {
      TransferSessionState.Created => WalletTransferStatus.waitingForScan,
      TransferSessionState.Paired => WalletTransferStatus.waitingForApprovalAndUpload,
      TransferSessionState.Confirmed => WalletTransferStatus.readyForTransferConfirmed,
      TransferSessionState.Uploaded => WalletTransferStatus.readyForDownload,
      TransferSessionState.Success => WalletTransferStatus.success,
      TransferSessionState.Canceled => WalletTransferStatus.cancelled,
      TransferSessionState.Error => WalletTransferStatus.error,
    };
  }

  @override
  Future<void> skipWalletTransfer() => _walletCore.skipWalletTransfer();
}
