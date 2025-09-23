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
  Future<void> acknowledgeWalletTransfer(String uri) => _walletCore.acknowledgeWalletTransfer(uri);

  @override
  Future<WalletInstructionResult> transferWallet(String pin) => _walletCore.transferWallet(pin);

  @override
  Future<void> cancelWalletTransfer() => _walletCore.cancelWalletTransfer();

  @override
  Future<WalletTransferStatus> getWalletTransferState() async {
    final result = await _walletCore.getWalletTransferState();
    return switch (result) {
      TransferSessionState.Created => WalletTransferStatus.waitingForScan,
      TransferSessionState.ReadyForTransfer => WalletTransferStatus.waitingForApproval,
      TransferSessionState.ReadyForDownload => WalletTransferStatus.transferring,
      TransferSessionState.Success => WalletTransferStatus.success,
      TransferSessionState.Cancelled => WalletTransferStatus.cancelled,
      TransferSessionState.Error => WalletTransferStatus.error,
    };
  }
}
