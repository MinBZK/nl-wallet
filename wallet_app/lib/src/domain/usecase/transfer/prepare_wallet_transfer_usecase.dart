import '../pin/check_pin_usecase.dart';

abstract class PrepareWalletTransferUseCase extends CheckPinUseCase {
  @override
  Future<Result<void>> invoke(String pin);
}
