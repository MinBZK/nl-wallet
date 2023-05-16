import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../../wallet_constants.dart';
import '../unlock_wallet_with_pin_usecase.dart';

class UnlockWalletWithPinUseCaseImpl extends UnlockWalletWithPinUseCase {
  final WalletRepository walletRepository;

  UnlockWalletWithPinUseCaseImpl(this.walletRepository);

  @override
  Future<bool> invoke(String pin) async {
    await Future.delayed(kDefaultMockDelay);
    await walletRepository.unlockWallet(pin);
    return await walletRepository.isLockedStream.first == false;
  }
}
