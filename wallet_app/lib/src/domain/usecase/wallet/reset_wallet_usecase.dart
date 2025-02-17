import '../wallet_usecase.dart';

abstract class ResetWalletUseCase extends WalletUseCase {
  Future<void> invoke();
}
