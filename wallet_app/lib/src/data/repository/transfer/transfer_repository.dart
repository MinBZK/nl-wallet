import 'package:wallet_core/core.dart';

import '../../../domain/model/transfer/wallet_transfer_status.dart';

abstract class TransferRepository {
  Future<String> initWalletTransfer();

  Future<void> acknowledgeWalletTransfer(String uri);

  Future<WalletInstructionResult> transferWallet(String pin);

  Future<void> cancelWalletTransfer();

  Future<WalletTransferStatus> getWalletTransferState();
}
