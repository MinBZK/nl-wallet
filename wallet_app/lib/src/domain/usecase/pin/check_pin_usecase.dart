import '../../model/pin/check_pin_result.dart';

export '../../model/pin/check_pin_result.dart';

abstract class CheckPinUseCase {
  Future<CheckPinResult> invoke(String pin);
}
