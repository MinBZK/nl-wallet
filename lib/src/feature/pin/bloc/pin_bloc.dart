import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/pin/base_check_pin_usecase.dart';
import '../../../domain/usecase/pin/get_available_pin_attempts_usecase.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_constants.dart';

part 'pin_event.dart';
part 'pin_state.dart';

class PinBloc extends Bloc<PinEvent, PinState> {
  final CheckPinUseCase checkPinUseCase;
  final GetAvailablePinAttemptsUseCase getAvailablePinAttemptsUseCase;

  String _currentPin = '';

  PinBloc(this.checkPinUseCase, this.getAvailablePinAttemptsUseCase) : super(const PinEntryInProgress(0)) {
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
    _currentPin = _currentPin.removeLastChar();
    emit(PinEntryInProgress(_currentPin.length));
  }

  Future<void> _validatePin(Emitter<PinState> emit) async {
    if (await checkPinUseCase.invoke(_currentPin)) {
      emit(const PinValidateSuccess());
    } else {
      _currentPin = '';
      int leftoverAttempts = getAvailablePinAttemptsUseCase.invoke();
      if (leftoverAttempts <= 0) {
        emit(const PinValidateBlocked());
      } else {
        emit(PinValidateFailure(leftoverAttempts));
      }
    }
  }
}
