import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/bloc/network_error_state.dart';
import '../../../domain/usecase/pin/check_pin_usecase.dart';
import '../../../util/extension/bloc_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_constants.dart';

part 'pin_event.dart';
part 'pin_state.dart';

class PinBloc extends Bloc<PinEvent, PinState> {
  final CheckPinUseCase checkPinUseCase;

  String _currentPin = '';

  String get currentPin => _currentPin;

  PinBloc(this.checkPinUseCase) : super(const PinEntryInProgress(0)) {
    on<PinDigitPressed>(_onEnterDigitEvent);
    on<PinBackspacePressed>(_onRemoveDigitEvent);
    on<PinClearPressed>(_onClearEvent);
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

  Future<void> _onClearEvent(event, emit) async {
    _currentPin = '';
    emit(const PinEntryInProgress(0, afterBackspacePressed: true));
  }

  Future<void> _validatePin(Emitter<PinState> emit) async {
    try {
      final result = await checkPinUseCase.invoke(_currentPin);
      if (result is! CheckPinResultOk) _currentPin = '';

      switch (result) {
        case CheckPinResultOk():
          emit(PinValidateSuccess(returnUrl: result.returnUrl));
        case CheckPinResultIncorrect():
          emit(
            PinValidateFailure(
              attemptsLeftInRound: result.attemptsLeftInRound,
              isFinalRound: result.isFinalRound,
            ),
          );
        case CheckPinResultTimeout():
          emit(PinValidateTimeout(DateTime.now().add(Duration(milliseconds: result.timeoutMillis))));
        case CheckPinResultBlocked():
          emit(const PinValidateBlocked());
      }
    } catch (ex) {
      Fimber.e('Pin validation error', ex: ex);
      await handleError(
        ex,
        onNetworkError: (ex, hasInternet) => emit(PinValidateNetworkError(error: ex, hasInternet: hasInternet)),
        onUnhandledError: (ex) => emit(PinValidateGenericError(error: ex)),
      );
      _currentPin = '';
    }
  }

  @override
  Future<dynamic> close() {
    _currentPin = '';
    return super.close();
  }
}
