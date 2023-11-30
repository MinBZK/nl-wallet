import '../pin/check_pin_usecase.dart';

abstract class AcceptSignAgreementUseCase extends CheckPinUseCase {
  @override
  Future<CheckPinResult> invoke(String pin);
}
