import '../../../data/repository/wallet/wallet_repository.dart';
import '../../../wallet_constants.dart';
import 'check_pin_usecase.dart';

class UnlockWalletWithPinUseCase extends CheckPinUseCase {
  final WalletRepository walletRepository;

  UnlockWalletWithPinUseCase(this.walletRepository);

  @override
  Future<bool> invoke(String pin) async {
    await Future.delayed(kDefaultMockDelay);
    walletRepository.unlockWallet(pin);
    return await walletRepository.isLockedStream.first == false;
  }
}
