import 'check_pin_usecase.dart';

abstract class UnlockWalletWithPinUseCase implements CheckPinUseCase {
  @override
  Future<bool> invoke(String pin);
}
