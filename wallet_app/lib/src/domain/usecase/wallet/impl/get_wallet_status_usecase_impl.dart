import 'package:fimber/fimber.dart';

import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../model/wallet_status.dart';
import '../get_wallet_status_usecase.dart';

class GetWalletStatusUseCaseImpl extends GetWalletStatusUseCase {
  final WalletRepository _walletRepository;

  GetWalletStatusUseCaseImpl(this._walletRepository);

  @override
  Future<WalletStatus> invoke() async {
    try {
      return await _walletRepository.getWalletStatus();
    } catch (ex) {
      Fimber.e('Failed to get wallet status', ex: ex);
      throw StateError('Failed to get wallet status');
    }
  }
}
