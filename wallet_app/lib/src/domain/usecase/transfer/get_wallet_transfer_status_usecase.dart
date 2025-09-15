import '../../model/transfer/wallet_transfer_status.dart';
import '../wallet_usecase.dart';

abstract class GetWalletTransferStatusUseCase extends WalletUseCase {
  Stream<WalletTransferStatus> invoke();
}
