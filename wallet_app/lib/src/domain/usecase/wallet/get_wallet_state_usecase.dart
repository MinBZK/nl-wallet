import '../../model/wallet_state.dart';
import '../wallet_usecase.dart';

abstract class GetWalletStateUseCase extends WalletUseCase {
  Future<WalletState> invoke();
}
