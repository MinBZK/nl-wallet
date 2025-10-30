import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/transfer/transfer_session_state.dart';
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
  Future<core.WalletInstructionResult> confirmWalletTransfer(String pin) => _walletCore.confirmWalletTransfer(pin);

  @override
  Future<void> transferWallet() => _walletCore.transferWallet();

  @override
  Future<void> receiveWalletTransfer() => _walletCore.receiveWalletTransfer();

  @override
  Future<void> cancelWalletTransfer() => _walletCore.cancelWalletTransfer();

  @override
  Future<TransferSessionState> getWalletTransferState() async {
    final result = await _walletCore.getWalletTransferState();
    return switch (result) {
      core.TransferSessionState.Created => TransferSessionState.created,
      core.TransferSessionState.Paired => TransferSessionState.paired,
      core.TransferSessionState.Confirmed => TransferSessionState.confirmed,
      core.TransferSessionState.Uploaded => TransferSessionState.uploaded,
      core.TransferSessionState.Success => TransferSessionState.success,
      core.TransferSessionState.Canceled => TransferSessionState.cancelled,
      core.TransferSessionState.Error => TransferSessionState.error,
    };
  }

  @override
  Future<void> skipWalletTransfer() => _walletCore.skipWalletTransfer();
}
