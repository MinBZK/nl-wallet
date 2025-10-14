import '../../../../data/repository/pin/pin_repository.dart';
import '../../../../util/extension/core_error_extension.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../model/pin/pin_validation_error.dart';
import '../../../model/result/application_error.dart';
import '../../../model/result/result.dart';
import '../check_is_valid_pin_usecase.dart';

class CheckIsValidPinUseCaseImpl extends CheckIsValidPinUseCase {
  final PinRepository _pinRepository;

  CheckIsValidPinUseCaseImpl(this._pinRepository);

  @override
  Future<Result<void>> invoke(String pin) async {
    try {
      await _pinRepository.validatePin(pin);
      return const Result.success(null);
    } on PinValidationError catch (ex) {
      return Result.error(ValidatePinError(ex, sourceError: ex));
    } on CoreError catch (ex) {
      return Result.error(await ex.asApplicationError());
    } catch (ex) {
      return Result.error(ValidatePinError(PinValidationError.other, sourceError: ex));
    }
  }
}
