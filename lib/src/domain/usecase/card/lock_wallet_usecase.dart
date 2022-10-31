import '../../../data/repository/wallet/wallet_repository.dart';

class LockWalletUseCase {
  final WalletRepository walletRepository;

  LockWalletUseCase(this.walletRepository);

  void lock() {
    walletRepository.lockWallet();
  }
}
