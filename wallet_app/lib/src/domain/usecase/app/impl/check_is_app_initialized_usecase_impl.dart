import '../../../../data/repository/wallet/wallet_repository.dart';
import '../check_is_app_initialized_usecase.dart';

class IsWalletInitializedUseCaseImpl implements IsWalletInitializedUseCase {
  final WalletRepository walletRepository;

  IsWalletInitializedUseCaseImpl(this.walletRepository);

  @override
  Future<bool> invoke() => walletRepository.isRegistered();
}
