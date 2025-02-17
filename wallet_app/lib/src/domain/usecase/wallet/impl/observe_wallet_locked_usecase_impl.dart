import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../wallet_usecase.dart';
import '../observe_wallet_locked_usecase.dart';

class ObserveWalletLockedUseCaseImpl extends ObserveWalletLockedUseCase {
  final WalletRepository _walletRepository;

  ObserveWalletLockedUseCaseImpl(this._walletRepository);

  @override
  Stream<bool> invoke() => _walletRepository.isLockedStream.handleAppError('Error while observing lock state');
}
