import '../pin/check_pin_usecase.dart';

export '../../model/pin/check_pin_result.dart';

abstract class AcceptOfferedPidUseCase implements CheckPinUseCase {
  @override
  Future<CheckPinResult> invoke(String pin);
}
