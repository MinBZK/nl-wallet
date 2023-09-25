import '../../../../data/repository/wallet/wallet_repository.dart';
import '../reset_wallet_usecase.dart';

class ResetWalletUseCaseImpl implements ResetWalletUseCase {
  final WalletRepository walletRepository;

  ResetWalletUseCaseImpl(this.walletRepository);

  @override
  Future<void> invoke() => walletRepository.resetWallet();
}
