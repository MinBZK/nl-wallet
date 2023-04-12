import '../../../../data/repository/wallet/wallet_repository.dart';
import '../check_is_app_initialized_usecase.dart';

class CheckIsAppInitializedUseCaseImpl implements CheckIsAppInitializedUseCase {
  final WalletRepository walletRepository;

  CheckIsAppInitializedUseCaseImpl(this.walletRepository);

  @override
  Future<bool> isInitialized() async {
    return walletRepository.isInitializedStream.first;
  }
}
