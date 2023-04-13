import '../../../../data/repository/wallet/wallet_repository.dart';
import '../lock_wallet_usecase.dart';

class LockWalletUseCaseImpl implements LockWalletUseCase {
  final WalletRepository walletRepository;

  LockWalletUseCaseImpl(this.walletRepository);

  @override
  void invoke() {
    walletRepository.lockWallet();
  }
}
