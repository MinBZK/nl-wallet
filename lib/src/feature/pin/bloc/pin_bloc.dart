import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/pin/get_available_pin_attempts_usecase.dart';
import '../../../domain/usecase/pin/unlock_wallet_usecase.dart';
import '../../../wallet_constants.dart';

part 'pin_event.dart';
part 'pin_state.dart';

class PinBloc extends Bloc<PinEvent, PinState> {
  final UnlockWalletUseCase unlockWalletUseCase;
  final GetAvailablePinAttemptsUseCase getAvailablePinAttemptsUseCase;

  String _currentPin = '';

  PinBloc(this.unlockWalletUseCase, this.getAvailablePinAttemptsUseCase) : super(const PinEntryInProgress(0)) {
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
    if (_currentPin.length == 1) {
      _currentPin = '';
    } else if (_currentPin.length > 1) {
      _currentPin = _currentPin.substring(0, _currentPin.length - 1);
    }
    emit(PinEntryInProgress(_currentPin.length));
  }

  Future<void> _validatePin(Emitter<PinState> emit) async {
    if (await unlockWalletUseCase.unlock(_currentPin)) {
      emit(const PinValidateSuccess());
    } else {
      _currentPin = '';
      int leftoverAttempts = await getAvailablePinAttemptsUseCase.getLeftoverAttempts();
      if (leftoverAttempts <= 0) {
        getAvailablePinAttemptsUseCase.reset();
        emit(const PinValidateBlocked());
      } else {
        emit(PinValidateFailure(leftoverAttempts));
      }
    }
  }
}
