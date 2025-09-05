import '../../../../data/repository/pin/pin_repository.dart';
import '../../../model/result/result.dart';
import '../create_pin_recovery_url_usecase.dart';

class CreatePinRecoveryRedirectUriUseCaseImpl extends CreatePinRecoveryRedirectUriUseCase {
  final PinRepository _pinRepository;

  CreatePinRecoveryRedirectUriUseCaseImpl(this._pinRepository);

  @override
  Future<Result<String>> invoke() {
    return tryCatch(
      () async => _pinRepository.createPinRecoveryRedirectUri(),
      'Failed to get pin recovery url',
    );
  }
}
