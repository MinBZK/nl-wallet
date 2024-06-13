import '../../../../data/repository/wallet/wallet_repository.dart';
import '../is_wallet_initialized_with_pid_usecase.dart';

class IsWalletInitializedWithPidUseCaseImpl implements IsWalletInitializedWithPidUseCase {
  final WalletRepository _walletRepository;

  IsWalletInitializedWithPidUseCaseImpl(this._walletRepository);

  @override
  Future<bool> invoke() async {
    final isInitialized = await _walletRepository.isRegistered();
    if (!isInitialized) return false;
    return _walletRepository.containsPid();
  }
}
