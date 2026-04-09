import '../pin/check_pin_usecase.dart';

/// Deletes a card from the wallet, requires PIN confirmation.
abstract class DeleteWalletCardUseCase extends CheckPinUseCase {
  @override
  Future<Result<void>> invoke(String pin);
}
