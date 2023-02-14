import '../../../data/repository/wallet/wallet_repository.dart';
import '../../../rust_core.dart';

class CreateWalletUseCase {
  final WalletRepository walletRepository;
  final RustCore rustCore;

  CreateWalletUseCase(this.walletRepository, this.rustCore);

  Future<bool> invoke(String pin) async {
    rustCore.register(pin: pin);
    return walletRepository.createWallet(pin);
  }
}
