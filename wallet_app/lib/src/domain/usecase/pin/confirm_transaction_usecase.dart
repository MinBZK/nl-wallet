import 'check_pin_usecase.dart';
export 'check_pin_usecase.dart';

abstract class ConfirmTransactionUseCase implements CheckPinUseCase {
  @override
  Future<CheckPinResult> invoke(String pin);
}
