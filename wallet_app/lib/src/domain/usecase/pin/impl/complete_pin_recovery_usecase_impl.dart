import '../../../../data/repository/pin/pin_repository.dart';
import '../../../model/result/result.dart';
import '../complete_pin_recovery_usecase.dart';

class CompletePinRecoveryUseCaseImpl extends CompletePinRecoveryUseCase {
  final PinRepository _pinRepository;

  CompletePinRecoveryUseCaseImpl(this._pinRepository);

  @override
  Future<Result<void>> invoke(String pin) {
    return tryCatch(
      () async => _pinRepository.completePinRecovery(pin),
      'Failed to complete pin recovery flow',
    );
  }
}
