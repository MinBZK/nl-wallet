import '../../../data/repository/wallet/wallet_repository.dart';

class CreateWalletUseCase {
  final WalletRepository walletRepository;

  CreateWalletUseCase(this.walletRepository);

  Future<bool> invoke(String pin) => walletRepository.createWallet(pin);
}
