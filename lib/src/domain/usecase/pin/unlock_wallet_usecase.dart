import '../../../data/repository/wallet/wallet_repository.dart';
import '../../../wallet_constants.dart';

class UnlockWalletUseCase {
  final WalletRepository walletRepository;

  UnlockWalletUseCase(this.walletRepository);

  Future<bool> unlock(String pin) async {
    await Future.delayed(kDefaultMockDelay);
    walletRepository.unlockWallet(pin);
    return await walletRepository.isLockedStream.first == false;
  }
}
