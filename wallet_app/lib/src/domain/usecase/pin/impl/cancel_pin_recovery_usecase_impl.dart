import '../../../../data/repository/pin/pin_repository.dart';
import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../../util/extension/wallet_state_extension.dart';
import '../../../model/result/result.dart';
import '../../../model/wallet_state.dart';
import '../cancel_pin_recovery_usecase.dart';

class CancelPinRecoveryUseCaseImpl extends CancelPinRecoveryUseCase {
  final PinRepository _pinRepository;
  final WalletRepository _walletRepository;

  CancelPinRecoveryUseCaseImpl(this._pinRepository, this._walletRepository);

  @override
  Future<Result<void>> invoke() {
    return tryCatch(
      () async {
        final WalletState state = await _walletRepository.getWalletState();
        if (state.unlockedState is WalletStateInPinRecoveryFlow) await _pinRepository.cancelPinRecovery();
      },
      'Failed to cancel pin recovery flow',
    );
  }
}
