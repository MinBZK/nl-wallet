import 'check_pin_usecase.dart';

abstract class ConfirmTransactionUseCase implements CheckPinUseCase {
  @override
  Future<bool> invoke(String pin);
}
