import '../../model/wallet_status.dart';
import '../wallet_usecase.dart';

abstract class GetWalletStatusUseCase extends WalletUseCase {
  Future<WalletStatus> invoke();
}
