import '../../../../data/repository/pin/pin_repository.dart';
import '../../../model/result/result.dart';
import '../continue_pin_recovery_usecase.dart';

class ContinuePinRecoveryUseCaseImpl extends ContinuePinRecoveryUseCase {
  final PinRepository _pinRepository;

  ContinuePinRecoveryUseCaseImpl(this._pinRepository);

  @override
  Future<Result<void>> invoke(String uri) {
    return tryCatch(
      () async => _pinRepository.continuePinRecovery(uri),
      'Failed to continue pin recovery flow',
    );
  }
}
