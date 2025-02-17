import 'package:fimber/fimber.dart';

import '../../../../data/repository/wallet/wallet_repository.dart';
import '../check_is_app_initialized_usecase.dart';

class IsWalletInitializedUseCaseImpl extends IsWalletInitializedUseCase {
  final WalletRepository walletRepository;

  IsWalletInitializedUseCaseImpl(this.walletRepository);

  @override
  Future<bool> invoke() async {
    try {
      return await walletRepository.isRegistered();
    } catch (exception) {
      Fimber.e('Failed to check registration state', ex: exception);
      throw StateError('Unable to check registration state');
    }
  }
}
