import 'package:wallet_core/core.dart';

import '../../../domain/model/transfer/wallet_transfer_status.dart';

abstract class TransferRepository {
  Future<String> initWalletTransfer();

  Future<void> pairWalletTransfer(String uri);

  Future<WalletInstructionResult> confirmWalletTransfer(String pin);

  Future<void> transferWallet();

  Future<void> receiveWalletTransfer();

  Future<void> cancelWalletTransfer();

  Future<WalletTransferStatus> getWalletTransferState();

  Future<void> skipWalletTransfer();
}
