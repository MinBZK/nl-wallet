import 'package:fimber/fimber.dart';

import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../model/wallet_state.dart';
import '../get_wallet_state_usecase.dart';

class GetWalletStateUseCaseImpl extends GetWalletStateUseCase {
  final WalletRepository _walletRepository;

  GetWalletStateUseCaseImpl(this._walletRepository);

  @override
  Future<WalletState> invoke() async {
    try {
      return await _walletRepository.getWalletState();
    } catch (ex) {
      Fimber.e('Failed to get wallet state', ex: ex);
      throw StateError('Failed to get wallet state');
    }
  }
}
