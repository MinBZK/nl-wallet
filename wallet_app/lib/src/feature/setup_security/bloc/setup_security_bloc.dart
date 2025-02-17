import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/bloc/network_error_state.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/pin/pin_validation_error.dart';
import '../../../domain/model/result/application_error.dart';
import '../../../domain/usecase/biometrics/get_available_biometrics_usecase.dart';
import '../../../domain/usecase/biometrics/set_biometrics_usecase.dart';
import '../../../domain/usecase/pin/check_is_valid_pin_usecase.dart';
import '../../../domain/usecase/wallet/create_wallet_usecase.dart';
import '../../../util/cast_util.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_constants.dart';

part 'setup_security_event.dart';
part 'setup_security_state.dart';

const int _kMaxConfirmAttempts = 1;

class SetupSecurityBloc extends Bloc<SetupSecurityEvent, SetupSecurityState> {
  final CheckIsValidPinUseCase checkIsValidPinUseCase;
  final CreateWalletUseCase createWalletUseCase;
  final GetAvailableBiometricsUseCase supportsBiometricsUsecase;
  final SetBiometricsUseCase setBiometricsUseCase;

  String _newPin = '';
  String _confirmPin = '';
  int _confirmAttempt = 0;

  SetupSecurityBloc(
    this.checkIsValidPinUseCase,
    this.createWalletUseCase,
    this.supportsBiometricsUsecase,
    this.setBiometricsUseCase,
  ) : super(const SetupSecuritySelectPinInProgress(0)) {
    on<SetupSecurityBackPressed>(_onSetupSecurityBackPressedEvent);
    on<PinDigitPressed>(_onPinDigitPressedEvent);
    on<PinBackspacePressed>(_onPinBackspacePressedEvent);
    on<PinClearPressed>(_onPinClearPressedEvent);
    on<SetupSecurityRetryPressed>(_onRetryPressed);
    on<EnableBiometricsPressed>(_onEnableBiometricsPressed);
    on<SkipBiometricsPressed>(_onSkipBiometricsPressed);
  }

  Future<void> _onSetupSecurityBackPressedEvent(event, emit) async {
    if (state.canGoBack) {
      if (state is SetupSecurityPinConfirmationInProgress) await _resetFlow(emit);
      if (state is SetupSecurityPinConfirmationFailed) await _resetFlow(emit);
    }
  }

  Future<void> _onPinDigitPressedEvent(event, emit) async {
    final state = this.state;
    if (state is SetupSecuritySelectPinInProgress || state is SetupSecuritySelectPinFailed) {
      await _onSelectPinDigitEvent(event, emit);
    }
    if (state is SetupSecurityPinConfirmationInProgress || state is SetupSecurityPinConfirmationFailed) {
      await _onConfirmPinDigitEvent(event, emit);
    }
  }

  Future<void> _onSelectPinDigitEvent(event, emit) async {
    _newPin += event.digit.toString();
    if (_newPin.length < kPinDigits) {
      emit(SetupSecuritySelectPinInProgress(_newPin.length));
      return; // User is still entering pin
    }
    assert(_newPin.length == kPinDigits, 'Unexpected pin length');

    final checkPinResult = await checkIsValidPinUseCase.invoke(_newPin);
    await checkPinResult.process(
      onSuccess: (_) => emit(const SetupSecurityPinConfirmationInProgress(0)),
      onError: (error) {
        _newPin = '';
        final reason = tryCast<ValidatePinError>(error)?.error ?? PinValidationError.other;
        emit(SetupSecuritySelectPinFailed(reason: reason));
      },
    );
  }

  Future<void> _onConfirmPinDigitEvent(event, emit) async {
    _confirmPin += event.digit.toString();
    if (_confirmPin.length != kPinDigits) {
      emit(SetupSecurityPinConfirmationInProgress(_confirmPin.length));
    } else {
      if (_newPin == _confirmPin) {
        emit(SetupSecurityCreatingWallet());
        await _createAndUnlockWallet(_newPin, emit);
      } else {
        _confirmPin = '';
        emit(SetupSecurityPinConfirmationFailed(retryAllowed: _confirmAttempt < _kMaxConfirmAttempts));
        _confirmAttempt++;
      }
    }
  }

  Future<void> _createAndUnlockWallet(String pin, emit) async {
    final result = await createWalletUseCase.invoke(pin);
    await result.process(
      onSuccess: (_) async {
        final biometrics = await supportsBiometricsUsecase.invoke();
        switch (biometrics) {
          case Biometrics.none:
            emit(const SetupSecurityCompleted());
          case Biometrics.face:
          case Biometrics.fingerprint:
          case Biometrics.some:
            emit(SetupSecurityConfigureBiometrics(biometrics: biometrics));
        }
      },
      onError: (error) async {
        switch (error) {
          case NetworkError():
            emit(SetupSecurityNetworkError(error: error, hasInternet: error.hasInternet));
          case HardwareUnsupportedError():
            emit(SetupSecurityDeviceIncompatibleError(error: error));
          default:
            emit(SetupSecurityGenericError(error: error));
        }
        await _resetFlow(emit);
      },
    );
  }

  Future<void> _onPinBackspacePressedEvent(event, emit) async {
    final state = this.state;
    if (state is SetupSecuritySelectPinInProgress || state is SetupSecuritySelectPinFailed) {
      await _onSelectPinBackspaceEvent(event, emit);
    }
    if (state is SetupSecurityPinConfirmationInProgress || state is SetupSecurityPinConfirmationFailed) {
      await _onConfirmPinBackspaceEvent(event, emit);
    }
  }

  Future<void> _onPinClearPressedEvent(event, emit) async {
    final state = this.state;
    if (state is SetupSecuritySelectPinInProgress || state is SetupSecuritySelectPinFailed) {
      _newPin = '';
      emit(const SetupSecuritySelectPinInProgress(0, afterBackspacePressed: true));
    }
    if (state is SetupSecurityPinConfirmationInProgress || state is SetupSecurityPinConfirmationFailed) {
      _confirmPin = '';
      emit(const SetupSecurityPinConfirmationInProgress(0, afterBackspacePressed: true));
    }
  }

  Future<void> _onSelectPinBackspaceEvent(event, emit) async {
    _newPin = _newPin.removeLastChar;
    emit(SetupSecuritySelectPinInProgress(_newPin.length, afterBackspacePressed: true));
  }

  Future<void> _onConfirmPinBackspaceEvent(event, emit) async {
    _confirmPin = _confirmPin.removeLastChar;
    emit(SetupSecurityPinConfirmationInProgress(_confirmPin.length, afterBackspacePressed: true));
  }

  Future<void> _onRetryPressed(event, emit) => _resetFlow(emit);

  Future<void> _onEnableBiometricsPressed(event, emit) async {
    assert(state is SetupSecurityConfigureBiometrics, 'Can only enable biometrics from the configuration state');
    final result = await setBiometricsUseCase.invoke(enable: true, authenticateBeforeEnabling: true);
    await result.process(
      onSuccess: (_) {
        final biometrics = tryCast<SetupSecurityConfigureBiometrics>(state)?.biometrics ?? Biometrics.some;
        emit(SetupSecurityCompleted(enabledBiometrics: biometrics));
      },
      onError: (error) => Fimber.e('Failed to enable biometrics', ex: error),
    );
  }

  Future<void> _onSkipBiometricsPressed(event, emit) async {
    assert(state is SetupSecurityConfigureBiometrics, 'Can only skip from the configuration state');
    emit(const SetupSecurityCompleted());
  }

  Future<void> _resetFlow(emit) async {
    _newPin = '';
    _confirmPin = '';
    _confirmAttempt = 0;
    emit(const SetupSecuritySelectPinInProgress(0, afterBackPressed: true));
  }
}
