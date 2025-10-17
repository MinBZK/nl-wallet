import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:wallet_core/core.dart';

import '../../../../data/repository/pin/pin_repository.dart';
import '../../../../util/extension/core_error_extension.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../model/result/application_error.dart';
import '../../../model/result/result.dart';
import '../change_pin_usecase.dart';

class ChangePinUseCaseImpl extends ChangePinUseCase {
  final PinRepository _pinRepository;

  ChangePinUseCaseImpl(this._pinRepository);

  @override
  Future<Result<void>> invoke(String oldPin, String newPin) async {
    bool pinUpdated = false;
    try {
      final result = await _pinRepository.changePin(oldPin, newPin);
      pinUpdated = result is WalletInstructionResult_Ok;
      return const Result.success(null);
    } on CoreError catch (ex) {
      Fimber.e('Error occurred while changing pin', ex: ex);
      return Result.error(await ex.asApplicationError());
    } catch (ex) {
      Fimber.e('Error occurred while changing pin', ex: ex);
      return Result.error(GenericError(ex.toString(), sourceError: ex));
    } finally {
      // As far as the user is concerned, the pin change is now complete. This sends an async ack to the server.
      unawaited(continueChangePin(pinUpdated ? newPin : oldPin));
    }
  }

  Future<void> continueChangePin(String pin) async {
    try {
      final result = await _pinRepository.continueChangePin(pin);
      switch (result) {
        case WalletInstructionResult_Ok():
          Fimber.d('Successfully notified server about commit/rollback');
        case WalletInstructionResult_InstructionError():
          Fimber.e('Failed to commit/rollback', ex: result);
      }
    } catch (ex) {
      Fimber.e('Failed to commit/rollback', ex: ex);
    }
  }
}
