import '../../../../data/repository/wallet/wallet_repository.dart';
import '../observe_wallet_lock_usecase.dart';

class ObserveWalletLockUseCaseImpl extends ObserveWalletLockUseCase {
  final WalletRepository _walletRepository;

  ObserveWalletLockUseCaseImpl(this._walletRepository);

  @override
  Stream<bool> invoke() => _walletRepository.isLockedStream;
}
