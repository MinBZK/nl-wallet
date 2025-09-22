import '../../model/transfer/wallet_transfer_status.dart';
import '../wallet_usecase.dart';

/// Use case for observing the status of a wallet transfer.
abstract class GetWalletTransferStatusUseCase extends WalletUseCase {
  Stream<WalletTransferStatus> invoke({
    bool isTarget = false /* used by mock */,
  });
}
