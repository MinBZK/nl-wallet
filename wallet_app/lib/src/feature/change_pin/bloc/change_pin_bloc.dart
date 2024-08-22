import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/bloc/network_error_state.dart';
import '../../../domain/model/pin/pin_validation_error.dart';
import '../../../domain/usecase/pin/change_pin_usecase.dart';
import '../../../domain/usecase/pin/check_is_valid_pin_usecase.dart';
import '../../../util/cast_util.dart';
import '../../../util/extension/bloc_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_constants.dart';

part 'change_pin_event.dart';
part 'change_pin_state.dart';

const int _kMaxConfirmAttempts = 1;

class ChangePinBloc extends Bloc<ChangePinEvent, ChangePinState> {
  final CheckIsValidPinUseCase checkIsValidPinUseCase;
  final ChangePinUseCase changePinUseCase;

  late String _currentPin;
  String _newPin = '';
  String _confirmNewPin = '';
  int _confirmAttempt = 0;

  bool get isEnteringNewPin => state is ChangePinSelectNewPinInProgress || state is ChangePinSelectNewPinFailed;

  bool get isConfirmingNewPin => state is ChangePinConfirmNewPinInProgress || state is ChangePinConfirmNewPinFailed;

  ChangePinBloc(this.checkIsValidPinUseCase, this.changePinUseCase) : super(const ChangePinInitial()) {
    on<ChangePinCurrentPinValidated>(_onCurrentPinValidated);
    on<ChangePinBackPressed>(_onBackPressedEvent);
    on<PinDigitPressed>(_onPinDigitPressedEvent);
    on<PinBackspacePressed>(_onPinBackspacePressedEvent);
    on<PinClearPressed>(_onPinClearPressedEvent);
    on<ChangePinRetryPressed>(_onRetryPressed);
  }

  Future<void> _onCurrentPinValidated(ChangePinCurrentPinValidated event, emit) async {
    _currentPin = event.currentPin;
    emit(const ChangePinSelectNewPinInProgress(0));
  }

  Future<void> _onBackPressedEvent(ChangePinBackPressed event, emit) async {
    if (isEnteringNewPin) {
      await _resetFlow(emit);
    } else if (isConfirmingNewPin) {
      add(ChangePinRetryPressed());
    }
  }

  Future<void> _onPinDigitPressedEvent(PinDigitPressed event, emit) async {
    if (_currentPin.length != kPinDigits) throw 'current pin should be available to setup a new pin';
    if (isEnteringNewPin) {
      await _onSelectNewPinDigitEvent(event, emit);
    } else if (isConfirmingNewPin) {
      await _onConfirmNewPinDigitEvent(event, emit);
    }
  }

  /// Handle events for when the user is selecting a new pin
  Future<void> _onSelectNewPinDigitEvent(event, emit) async {
    _newPin += event.digit.toString();
    if (_newPin.length == kPinDigits) {
      try {
        await checkIsValidPinUseCase.invoke(_newPin);
        emit(const ChangePinConfirmNewPinInProgress(0));
      } catch (error) {
        _newPin = '';
        emit(ChangePinSelectNewPinFailed(reason: tryCast<PinValidationError>(error) ?? PinValidationError.other));
      }
    } else {
      emit(ChangePinSelectNewPinInProgress(_newPin.length));
    }
  }

  /// Handle events for when the user is confirming the new pin
  Future<void> _onConfirmNewPinDigitEvent(event, emit) async {
    if (_newPin.length != kPinDigits) throw 'new pin should already be provided once';
    _confirmNewPin += event.digit.toString();
    if (_confirmNewPin.length != kPinDigits) {
      emit(ChangePinConfirmNewPinInProgress(_confirmNewPin.length));
    } else {
      if (_newPin == _confirmNewPin) {
        await _changePin(emit);
      } else {
        _confirmNewPin = '';
        emit(ChangePinConfirmNewPinFailed(retryAllowed: _confirmAttempt < _kMaxConfirmAttempts));
        _confirmAttempt++;
      }
    }
  }

  /// Updates the PIN from [_currentPin] to [_newPin]
  Future<void> _changePin(emit) async {
    assert(_currentPin.length == kPinDigits, 'Current pin unavailable');
    assert(_newPin.length == kPinDigits, 'New pin unavailable');
    try {
      emit(ChangePinUpdating());
      await changePinUseCase.invoke(_currentPin, _newPin);
      emit(ChangePinCompleted());
    } catch (ex) {
      Fimber.e('Failed to update pin', ex: ex);
      await handleError(
        ex,
        onNetworkError: (ex, hasInternet) => emit(ChangePinNetworkError(error: ex, hasInternet: hasInternet)),
        onUnhandledError: (ex) => emit(ChangePinGenericError(error: ex)),
      );
      await _resetFlow(emit);
    }
  }

  Future<void> _onPinBackspacePressedEvent(PinBackspacePressed event, emit) async {
    if (isEnteringNewPin) {
      await _onSelectNewPinBackspaceEvent(event, emit);
    }
    if (isConfirmingNewPin) {
      await _onConfirmNewPinBackspaceEvent(event, emit);
    }
  }

  Future<void> _onSelectNewPinBackspaceEvent(event, emit) async {
    _newPin = _newPin.removeLastChar;
    emit(ChangePinSelectNewPinInProgress(_newPin.length, afterBackspacePressed: true));
  }

  Future<void> _onConfirmNewPinBackspaceEvent(event, emit) async {
    _confirmNewPin = _confirmNewPin.removeLastChar;
    emit(ChangePinConfirmNewPinInProgress(_confirmNewPin.length, afterBackspacePressed: true));
  }

  Future<void> _onPinClearPressedEvent(PinClearPressed event, emit) async {
    if (isEnteringNewPin) {
      _newPin = '';
      emit(const ChangePinSelectNewPinInProgress(0, afterBackspacePressed: true));
    }
    if (isConfirmingNewPin) {
      _confirmNewPin = '';
      emit(const ChangePinConfirmNewPinInProgress(0, afterBackspacePressed: true));
    }
  }

  // Resets the BLoC to the 'enter new pin' state (i.e. after confirming current pin).
  Future<void> _onRetryPressed(ChangePinRetryPressed event, emit) async {
    if (_currentPin.length != kPinDigits) throw 'current pin should be available to setup a new pin';
    _newPin = '';
    _confirmNewPin = '';
    _confirmAttempt = 0;
    emit(const ChangePinSelectNewPinInProgress(0, didGoBack: true));
  }

  /// Resets the BLoC to it's initial state
  Future<void> _resetFlow(emit) async {
    _currentPin = '';
    _newPin = '';
    _confirmNewPin = '';
    _confirmAttempt = 0;
    emit(const ChangePinInitial(didGoBack: true));
  }

  @override
  Future<void> close() {
    // Clean up any references to the pin
    _currentPin = '';
    _newPin = '';
    _confirmNewPin = '';
    return super.close();
  }
}
