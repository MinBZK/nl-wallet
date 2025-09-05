import 'dart:async';
import 'dart:math' as math;

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/bloc/network_error_state.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/pin/pin_validation_error.dart';
import '../../../domain/model/result/application_error.dart';
import '../../../domain/usecase/pin/cancel_pin_recovery_usecase.dart';
import '../../../domain/usecase/pin/check_is_valid_pin_usecase.dart';
import '../../../domain/usecase/pin/complete_pin_recovery_usecase.dart';
import '../../../domain/usecase/pin/continue_pin_recovery_usecase.dart';
import '../../../domain/usecase/pin/create_pin_recovery_url_usecase.dart';
import '../../../util/cast_util.dart';
import '../../../wallet_constants.dart';
import '../../../wallet_core/error/core_error.dart';

part 'recover_pin_event.dart';
part 'recover_pin_state.dart';

class RecoverPinBloc extends Bloc<RecoverPinEvent, RecoverPinState> {
  final CreatePinRecoveryRedirectUriUseCase _createPinRecoveryRedirectUriUsecase;
  final CheckIsValidPinUseCase _checkIsValidPinUseCase;
  final ContinuePinRecoveryUseCase _continuePinRecoveryUsecase;
  final CancelPinRecoveryUseCase _cancelPinRecoveryUsecase;
  final CompletePinRecoveryUseCase _completePinRecoveryUsecase;

  RecoverPinBloc(
    this._createPinRecoveryRedirectUriUsecase,
    this._checkIsValidPinUseCase,
    this._continuePinRecoveryUsecase,
    this._cancelPinRecoveryUsecase,
    this._completePinRecoveryUsecase, {
    required bool continueFromDigiD,
  }) : super(continueFromDigiD ? const RecoverPinVerifyingDigidAuthentication() : const RecoverPinInitial()) {
    on<RecoverPinLoginWithDigidClicked>(_onDigidLoginClicked);
    on<RecoverPinLoginWithDigidFailed>(_onDigidLoginFailed);
    on<RecoverPinContinuePinRecovery>(_onContinuePinRecovery);

    // Listeners to handle pin keyboard entry
    on<RecoverPinDigitPressed>(_onPinDigitPressed);
    on<RecoverPinBackspacePressed>(_onPinBackspacePressed);
    on<RecoverPinClearPressed>(_onPinClearPressed);

    on<RecoverPinBackPressed>(_onBackPressed);
    on<RecoverPinRetryPressed>(_onRetryPressed);
    on<RecoverPinStopPressed>(_onStopPressed);
  }

  FutureOr<void> _onDigidLoginClicked(RecoverPinLoginWithDigidClicked event, Emitter<RecoverPinState> emit) async {
    emit(const RecoverPinLoadingDigidUrl());
    final result = await _createPinRecoveryRedirectUriUsecase.invoke();
    await result.process(
      onSuccess: (url) => emit(RecoverPinAwaitingDigidAuthentication(url)),
      onError: (error) async => _handleApplicationError(error, emit),
    );
  }

  FutureOr<void> _onContinuePinRecovery(RecoverPinContinuePinRecovery event, Emitter<RecoverPinState> emit) async {
    emit(const RecoverPinVerifyingDigidAuthentication());
    final result = await _continuePinRecoveryUsecase.invoke(event.authUrl);
    await result.process(
      onSuccess: (_) => emit(RecoverPinChooseNewPin(authUrl: event.authUrl)),
      onError: (error) async => _handleApplicationError(error, emit),
    );
  }

  Future<void> _completePinRecovery(String pin, Emitter<RecoverPinState> emit) async {
    emit(const RecoverPinUpdatingPin());
    final result = await _completePinRecoveryUsecase.invoke(pin);
    await result.process(
      onSuccess: (_) => emit(const RecoverPinSuccess()),
      onError: (error) async => _handleApplicationError(error, emit),
    );
  }

  FutureOr<void> _onBackPressed(RecoverPinBackPressed event, Emitter<RecoverPinState> emit) async {
    final state = this.state;
    if (!state.canGoBack) return;
    if (state is RecoverPinConfirmNewPin) emit(RecoverPinChooseNewPin(didGoBack: true, authUrl: state.authUrl));
    if (state is RecoverPinChooseNewPin) emit(const RecoverPinInitial(didGoBack: true));
  }

  FutureOr<void> _onDigidLoginFailed(RecoverPinLoginWithDigidFailed event, Emitter<RecoverPinState> emit) async {
    if (event.cancelledByUser) {
      emit(const RecoverPinDigidLoginCancelled());
    } else {
      await _handleApplicationError(event.error, emit);
    }
  }

  FutureOr<void> _onRetryPressed(RecoverPinRetryPressed event, Emitter<RecoverPinState> emit) {
    add(const RecoverPinLoginWithDigidClicked());
  }

  FutureOr<void> _onStopPressed(RecoverPinStopPressed event, Emitter<RecoverPinState> emit) async {
    await _cancelPinRecoveryUsecase.invoke();
    emit(const RecoverPinStopped());
  }

  FutureOr<void> _onPinDigitPressed(RecoverPinDigitPressed event, Emitter<RecoverPinState> emit) async {
    final state = this.state;
    if (state is RecoverPinChooseNewPin) await _processPinDigit(state, event.digit, emit);
    if (state is RecoverPinConfirmNewPin) await _processConfirmPinDigit(state, event.digit, emit);
  }

  FutureOr<void> _processPinDigit(
    RecoverPinChooseNewPin state,
    int digit,
    Emitter<RecoverPinState> emit,
  ) async {
    final enteredPin = '${state.pin}$digit';
    if (enteredPin.length < kPinDigits) {
      emit(RecoverPinChooseNewPin(authUrl: state.authUrl, pin: enteredPin));
    } else if (enteredPin.length == kPinDigits) {
      final pinValidationResult = await _checkIsValidPinUseCase.invoke(enteredPin);
      await pinValidationResult.process(
        onSuccess: (success) =>
            emit(RecoverPinConfirmNewPin(authUrl: state.authUrl, selectedPin: enteredPin, isRetrying: false)),
        onError: (error) async {
          if (error is ValidatePinError) {
            emit(RecoverPinSelectPinFailed(error: error));
            emit(RecoverPinChooseNewPin(authUrl: state.authUrl));
          } else {
            await _handleApplicationError(error, emit);
          }
        },
      );
    }
  }

  FutureOr<void> _processConfirmPinDigit(
    RecoverPinConfirmNewPin state,
    int digit,
    Emitter<RecoverPinState> emit,
  ) async {
    final pin = '${state.pin}$digit';
    if (pin.length < kPinDigits) {
      emit(state.copyWith(pin: pin));
    } else if (pin.length == kPinDigits) {
      if (state.selectedPin == pin) {
        // User confirmed the pin, update it
        await _completePinRecovery(pin, emit);
      } else {
        const pinMismatchError = GenericError('pin_mismatch', sourceError: 'bloc');
        emit(RecoverPinConfirmPinFailed(error: pinMismatchError, canRetry: !state.isRetrying));
        if (state.isRetrying) {
          // User failed to confirm pin twice, and should select a new pin
          emit(RecoverPinChooseNewPin(authUrl: state.authUrl));
        } else {
          // User failed to confirm pin, but can retry once
          emit(state.copyWith(pin: '', isRetrying: true));
        }
      }
    }
  }

  FutureOr<void> _onPinBackspacePressed(RecoverPinBackspacePressed event, Emitter<RecoverPinState> emit) {
    final state = this.state;
    if (state is RecoverPinChooseNewPin) {
      final newPinLength = math.max(0, state.pin.length - 1);
      final updatedPin = state.pin.substring(0, newPinLength);
      emit(state.copyWith(pin: updatedPin));
    }
    if (state is RecoverPinConfirmNewPin) {
      final newPinLength = math.max(0, state.pin.length - 1);
      final updatedPin = state.pin.substring(0, newPinLength);
      emit(state.copyWith(pin: updatedPin));
    }
  }

  FutureOr<void> _onPinClearPressed(RecoverPinClearPressed event, Emitter<RecoverPinState> emit) {
    final state = this.state;
    if (state is RecoverPinChooseNewPin) emit(state.copyWith(pin: ''));
    if (state is RecoverPinConfirmNewPin) emit(state.copyWith(pin: ''));
  }

  Future<void> _handleApplicationError(ApplicationError error, Emitter<RecoverPinState> emit) async {
    await _cancelPinRecoveryUsecase.invoke(); // Always attempt to cancel the session, then render the specific error

    // TODO(Rob): Handle DigiDMismatch and emit [RecoverPinDigidMismatch]
    switch (error) {
      case NetworkError():
        emit(RecoverPinNetworkError(hasInternet: error.hasInternet, error: error));
      case RedirectUriError():
        if ([RedirectError.accessDenied, RedirectError.loginRequired].contains(error.redirectError)) {
          emit(const RecoverPinDigidLoginCancelled());
        } else {
          emit(RecoverPinDigidFailure(error: error));
        }
      case SessionError():
        emit(RecoverPinSessionExpired(error: error));
      default:
        Fimber.w('Handling ${error.runtimeType} as generic error.', ex: error);
        emit(RecoverPinGenericError(error: error));
    }
  }

  @override
  Future<void> close() {
    _cancelPinRecoveryUsecase.invoke();
    return super.close();
  }
}
