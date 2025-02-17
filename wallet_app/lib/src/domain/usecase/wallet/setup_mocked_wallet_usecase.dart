import '../wallet_usecase.dart';

abstract class SetupMockedWalletUseCase extends WalletUseCase {
  Future<void> invoke();
}
