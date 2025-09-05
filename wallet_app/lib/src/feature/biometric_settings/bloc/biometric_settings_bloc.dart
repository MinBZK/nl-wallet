import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/biometrics/biometric_authentication_result.dart';
import '../../../domain/usecase/biometrics/get_available_biometrics_usecase.dart';
import '../../../domain/usecase/biometrics/get_supported_biometrics_usecase.dart';
import '../../../domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';
import '../../../domain/usecase/biometrics/request_biometrics_usecase.dart';
import '../../../domain/usecase/biometrics/set_biometrics_usecase.dart';

part 'biometric_settings_event.dart';
part 'biometric_settings_state.dart';

class BiometricSettingsBloc extends Bloc<BiometricSettingsEvent, BiometricSettingsState> {
  final GetSupportedBiometricsUseCase getSupportedBiometricsUseCase;
  final GetAvailableBiometricsUseCase getAvailableBiometricsUseCase;
  final SetBiometricsUseCase setBiometricsUseCase;
  final IsBiometricLoginEnabledUseCase isBiometricLoginEnabledUseCase;
  final RequestBiometricsUseCase requestBiometricsUsecase;

  Biometrics? _supportedBiometrics;

  /// Expose supported biometrics with sane default, useful to render correct title in all available states.
  Biometrics get supportedBiometrics => _supportedBiometrics ?? Biometrics.some;

  BiometricSettingsBloc(
    this.getSupportedBiometricsUseCase,
    this.getAvailableBiometricsUseCase,
    this.setBiometricsUseCase,
    this.isBiometricLoginEnabledUseCase,
    this.requestBiometricsUsecase,
  ) : super(BiometricSettingsInitial()) {
    on<BiometricLoadTriggered>(_onRefresh);
    on<BiometricUnlockToggled>(_onBiometricUnlockSettingToggled);
    on<BiometricUnlockEnabledWithPin>(_onBiometricUnlockEnabled);
  }

  Future<void> _onRefresh(BiometricLoadTriggered event, Emitter<BiometricSettingsState> emit) async {
    try {
      _supportedBiometrics ??= await getSupportedBiometricsUseCase.invoke();
      final state = await isBiometricLoginEnabledUseCase.invoke();
      emit(BiometricSettingsLoaded(biometricLoginEnabled: state));
    } catch (ex) {
      Fimber.e('Failed to get state', ex: ex);
      emit(const BiometricSettingsError());
    }
  }

  Future<void> _onBiometricUnlockSettingToggled(
    BiometricUnlockToggled event,
    Emitter<BiometricSettingsState> emit,
  ) async {
    final localState = state;
    if (localState is BiometricSettingsLoaded) {
      if (localState.biometricLoginEnabled) {
        await _disableBiometrics(localState, emit);
      } else {
        await _enableBiometrics(localState, emit);
      }
    } else {
      Fimber.e("Can't toggle biometric unlock while in state: $state");
    }
  }

  Future<void> _disableBiometrics(BiometricSettingsLoaded currentState, Emitter<BiometricSettingsState> emit) async {
    emit(const BiometricSettingsLoaded(biometricLoginEnabled: false));
    final result = await setBiometricsUseCase.invoke(enable: false, authenticateBeforeEnabling: false);
    await result.process(
      onSuccess: (_) {},
      onError: (error) {
        Fimber.e('Failed to disable biometrics', ex: error);
        emit(currentState); // Revert state changes
      },
    );
  }

  Future<void> _enableBiometrics(BiometricSettingsLoaded currentState, Emitter<BiometricSettingsState> emit) async {
    // Eagerly update the UI
    emit(const BiometricSettingsLoaded(biometricLoginEnabled: true));

    // Perform biometric authentication
    final result = await requestBiometricsUsecase.invoke();
    await result.process(
      onSuccess: (authResult) {
        switch (authResult) {
          case BiometricAuthenticationResult.success:
            emit(const BiometricSettingsConfirmPin()); // Request PIN to confirm
          case BiometricAuthenticationResult.lockedOut:
            emit(const BiometricSettingsLockedOut());
            emit(currentState);
          case BiometricAuthenticationResult.setupRequired:
            emit(const BiometricSettingsSetupRequired());
            emit(currentState);
        }
      },
      onError: (error) {
        Fimber.e('Could not enable biometrics', ex: error);
        emit(currentState);
      },
    );
  }

  Future<void> _onBiometricUnlockEnabled(
    BiometricUnlockEnabledWithPin event,
    Emitter<BiometricSettingsState> emit,
  ) async {
    final availability = await getAvailableBiometricsUseCase.invoke();
    if (availability == Biometrics.none) {
      Fimber.e('Biometrics can only be enabled if the device supports it.');
      emit(const BiometricSettingsError());
      return;
    }
    final result = await setBiometricsUseCase.invoke(enable: true, authenticateBeforeEnabling: false);
    await result.process(
      onSuccess: (_) => emit(const BiometricSettingsLoaded(biometricLoginEnabled: true)),
      onError: (error) => Fimber.e('Failed to enable biometric unlock', ex: error),
    );
  }
}
