import 'package:fimber/fimber.dart';

import '../../../../data/repository/pin/pin_repository.dart';
import '../../../../util/extension/core_error_extension.dart';
import '../../../../util/extension/wallet_instruction_result_extension.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../model/result/application_error.dart';
import '../unlock_wallet_with_pin_usecase.dart';

// Checks if the provided pin matches the registered pin
class CheckPinUseCaseImpl extends CheckPinUseCase {
  final PinRepository _pinRepository;

  CheckPinUseCaseImpl(this._pinRepository);

  @override
  Future<Result<String?>> invoke(String pin) async {
    try {
      final result = await _pinRepository.checkPin(pin);
      return result.asApplicationResult();
    } on CoreError catch (ex) {
      Fimber.e('Failed to check pin', ex: ex);
      return Result.error(await ex.asApplicationError());
    } catch (ex) {
      Fimber.e('Failed to check pin', ex: ex);
      return Result.error(GenericError(ex.toString(), sourceError: ex));
    }
  }
}
