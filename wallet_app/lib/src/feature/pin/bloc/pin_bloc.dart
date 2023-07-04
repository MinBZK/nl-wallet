import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/pin/check_pin_usecase.dart';
import '../../../util/extension/check_pin_result_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_constants.dart';

part 'pin_event.dart';
part 'pin_state.dart';

class PinBloc extends Bloc<PinEvent, PinState> {
  final CheckPinUseCase checkPinUseCase;

  String _currentPin = '';

  PinBloc(this.checkPinUseCase) : super(const PinEntryInProgress(0)) {
    on<PinDigitPressed>(_onEnterDigitEvent);
    on<PinBackspacePressed>(_onRemoveDigitEvent);
  }

  FutureOr<void> _onEnterDigitEvent(event, emit) async {
    if (state is PinValidateInProgress) return;
    _currentPin += event.digit.toString();
    if (_currentPin.length == kPinDigits) {
      emit(const PinValidateInProgress());
      await _validatePin(emit);
    } else {
      emit(PinEntryInProgress(_currentPin.length));
    }
  }

  FutureOr<void> _onRemoveDigitEvent(event, emit) {
    _currentPin = _currentPin.removeLastChar;
    emit(PinEntryInProgress(_currentPin.length, afterBackspacePressed: true));
  }

  Future<void> _validatePin(Emitter<PinState> emit) async {
    final checkPinResult = await checkPinUseCase.invoke(_currentPin);
    if (checkPinResult is! CheckPinResultOk) _currentPin = '';
    checkPinResult.when(
      onCheckPinResultOk: (it) => emit(const PinValidateSuccess()),
      onCheckPinResultIncorrectPin: (it) =>
          emit(PinValidateFailure(leftoverAttempts: it.leftoverAttempts, isFinalAttempt: it.isFinalAttempt)),
      onCheckPinResultTimeout: (it) =>
          emit(PinValidateTimeout(DateTime.now().add(Duration(milliseconds: it.timeoutMillis)))),
      onCheckPinResultBlocked: (it) => emit(const PinValidateBlocked()),
      onCheckPinResultServerError: (it) => emit(const PinValidateServerError()),
    );
  }
}
