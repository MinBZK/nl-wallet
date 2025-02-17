import '../wallet_usecase.dart';

abstract class LockWalletUseCase extends WalletUseCase {
  Future<void> invoke();
}
