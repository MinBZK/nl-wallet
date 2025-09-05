import '../../../../data/repository/pin/pin_repository.dart';
import '../../../model/result/result.dart';
import '../cancel_pin_recovery_usecase.dart';

class CancelPinRecoveryUseCaseImpl extends CancelPinRecoveryUseCase {
  final PinRepository _pinRepository;

  CancelPinRecoveryUseCaseImpl(this._pinRepository);

  @override
  Future<Result<void>> invoke() {
    return tryCatch(
      () async => _pinRepository.cancelPinRecovery(),
      'Failed to cancel pin recovery flow',
    );
  }
}
