import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/transfer/transfer_session_state.dart';

abstract class TransferRepository {
  Future<String> initWalletTransfer();

  Future<void> pairWalletTransfer(String uri);

  Future<core.WalletInstructionResult> confirmWalletTransfer(String pin);

  Future<void> transferWallet();

  Future<void> receiveWalletTransfer();

  Future<void> cancelWalletTransfer();

  Future<TransferSessionState> getWalletTransferState();

  Future<void> skipWalletTransfer();
}
