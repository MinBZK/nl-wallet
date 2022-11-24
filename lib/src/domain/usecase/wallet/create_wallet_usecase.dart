import '../../../data/repository/wallet/wallet_repository.dart';
import '../../../wallet_constants.dart';

class CreateWalletUseCase {
  final WalletRepository walletRepository;

  CreateWalletUseCase(this.walletRepository);

  Future<bool> invoke(String pin) async {
    await Future.delayed(kDefaultMockDelay);
    return walletRepository.createWallet(pin);
  }
}
