import 'package:fimber/fimber.dart';

import '../../../../data/repository/wallet/wallet_repository.dart';
import '../is_wallet_initialized_with_pid_usecase.dart';

class IsWalletInitializedWithPidUseCaseImpl extends IsWalletInitializedWithPidUseCase {
  final WalletRepository _walletRepository;

  IsWalletInitializedWithPidUseCaseImpl(this._walletRepository);

  @override
  Future<bool> invoke() async {
    try {
      final isInitialized = await _walletRepository.isRegistered();
      if (!isInitialized) return false;
      return await _walletRepository.containsPid();
    } catch (exception) {
      Fimber.e('Failed to check if PID is available', ex: exception);
      throw StateError('Unable to check if pid exists');
    }
  }
}
