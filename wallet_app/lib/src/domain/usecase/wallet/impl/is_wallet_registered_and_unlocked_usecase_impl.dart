import 'package:fimber/fimber.dart';

import '../../../../data/repository/wallet/wallet_repository.dart';
import '../is_wallet_registered_and_unlocked_usecase.dart';

class IsWalletRegisteredAndUnlockedUseCaseImpl extends IsWalletRegisteredAndUnlockedUseCase {
  final WalletRepository _walletRepository;

  IsWalletRegisteredAndUnlockedUseCaseImpl(this._walletRepository);

  @override
  Future<bool> invoke() async {
    try {
      final registered = await _walletRepository.isRegistered();
      final locked = await _walletRepository.isLockedStream.first;
      return registered && !locked;
    } catch (ex) {
      Fimber.e('Failed to check for registration and locked state', ex: ex);
      throw StateError('Could not check wallet registration & locked state');
    }
  }
}
