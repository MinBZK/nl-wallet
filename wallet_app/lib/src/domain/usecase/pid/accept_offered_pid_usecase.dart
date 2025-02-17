import '../pin/check_pin_usecase.dart';

export '../../model/pin/check_pin_result.dart';

abstract class AcceptOfferedPidUseCase extends CheckPinUseCase {
  @override
  Future<Result<String?>> invoke(String pin);
}
