import 'package:fimber/fimber.dart';

import '../../../../data/repository/wallet/wallet_repository.dart';
import '../reset_wallet_usecase.dart';

class ResetWalletUseCaseImpl extends ResetWalletUseCase {
  final WalletRepository walletRepository;

  ResetWalletUseCaseImpl(this.walletRepository);

  @override
  Future<void> invoke() async {
    try {
      await walletRepository.resetWallet();
    } catch (ex) {
      Fimber.e('Failed to reset wallet', ex: ex);
      throw StateError('Failed to reset wallet');
    }
  }
}
