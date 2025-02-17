import 'check_pin_usecase.dart';

export 'check_pin_usecase.dart';

abstract class DiscloseForIssuanceUseCase extends CheckPinUseCase {
  @override
  Future<Result<String?>> invoke(String pin);
}
