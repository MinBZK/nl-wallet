import 'package:fimber/fimber.dart';

import '../../../../data/repository/wallet/wallet_repository.dart';
import '../lock_wallet_usecase.dart';

class LockWalletUseCaseImpl extends LockWalletUseCase {
  final WalletRepository _walletRepository;

  LockWalletUseCaseImpl(this._walletRepository);

  @override
  Future<void> invoke() async {
    try {
      await _walletRepository.lockWallet();
    } catch (ex) {
      Fimber.e('Failed to lock wallet', ex: ex);
      throw StateError('Could not lock wallet');
    }
  }
}
