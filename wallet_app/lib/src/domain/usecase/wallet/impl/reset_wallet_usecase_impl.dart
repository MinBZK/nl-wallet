import 'package:fimber/fimber.dart';

import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../../data/store/shared_preferences_provider.dart';
import '../reset_wallet_usecase.dart';

class ResetWalletUseCaseImpl extends ResetWalletUseCase {
  final WalletRepository _walletRepository;
  final PreferenceProvider _preferences;

  ResetWalletUseCaseImpl(this._walletRepository, this._preferences);

  @override
  Future<void> invoke() async {
    try {
      await _walletRepository.resetWallet();
      await (await _preferences()).clear();
    } catch (ex) {
      Fimber.e('Failed to reset wallet', ex: ex);
      throw StateError('Failed to reset wallet');
    }
  }
}
