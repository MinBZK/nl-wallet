import '../wallet_usecase.dart';

abstract class ObserveWalletLockedUseCase extends WalletUseCase {
  /// Stream exposes [true] when the wallet is currently locked.
  Stream<bool> invoke();
}
