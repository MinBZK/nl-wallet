import '../../../../data/repository/wallet/wallet_repository.dart';
import '../create_wallet_usecase.dart';

class CreateWalletUseCaseImpl implements CreateWalletUseCase {
  final WalletRepository walletRepository;

  CreateWalletUseCaseImpl(this.walletRepository);

  @override
  Future<void> invoke(String pin) => walletRepository.createWallet(pin);
}
