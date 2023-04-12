import 'package:fimber/fimber.dart';

import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../model/pin/pin_validation_error.dart';
import '../check_is_valid_pin_usecase.dart';

class CheckIsValidPinUseCaseImpl extends CheckIsValidPinUseCase {
  final WalletRepository _walletRepository;

  CheckIsValidPinUseCaseImpl(this._walletRepository);

  @override
  Future<void> invoke(String pin) async {
    try {
      await _walletRepository.validatePin(pin);
    } catch (error) {
      // Guarantee ONLY [PinValidationError]s are thrown
      if (error is PinValidationError) {
        rethrow;
      } else {
        Fimber.e('Something other than pin validation failed.', ex: error);
        throw PinValidationError.other;
      }
    }
  }
}
