import 'package:fimber/fimber.dart';

import '../../../../data/repository/tour/tour_repository.dart';
import '../../../../data/repository/wallet/wallet_repository.dart';
import '../reset_wallet_usecase.dart';

class ResetWalletUseCaseImpl extends ResetWalletUseCase {
  final WalletRepository _walletRepository;
  final TourRepository _tourRepository;

  ResetWalletUseCaseImpl(this._walletRepository, this._tourRepository);

  @override
  Future<void> invoke() async {
    try {
      await _walletRepository.resetWallet();
      await _tourRepository.setShowTourBanner(showTourBanner: true);
    } catch (ex) {
      Fimber.e('Failed to reset wallet', ex: ex);
      throw StateError('Failed to reset wallet');
    }
  }
}
