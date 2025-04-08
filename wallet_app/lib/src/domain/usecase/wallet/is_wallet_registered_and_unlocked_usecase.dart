import '../wallet_usecase.dart';

abstract class IsWalletRegisteredAndUnlockedUseCase extends WalletUseCase {
  Future<bool> invoke();
}
