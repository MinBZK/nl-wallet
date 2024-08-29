import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

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
  final RequestBiometricsUsecase requestBiometricsUsecaseImpl;

  Biometrics? _supportedBiometrics;

  /// Expose supported biometrics with sane default, useful to render correct title in all available states.
  Biometrics get supportedBiometrics => _supportedBiometrics ?? Biometrics.some;

  BiometricSettingsBloc(
    this.getSupportedBiometricsUseCase,
    this.getAvailableBiometricsUseCase,
    this.setBiometricsUseCase,
    this.isBiometricLoginEnabledUseCase,
    this.requestBiometricsUsecaseImpl,
  ) : super(BiometricSettingsInitial()) {
    on<BiometricLoadTriggered>(_onRefresh);
    on<BiometricUnlockToggled>(_onBiometricUnlockSettingToggled);
    on<BiometricUnlockEnabledWithPin>(_onBiometricUnlockEnabled);
  }

  Future<void> _onRefresh(event, emit) async {
    try {
      _supportedBiometrics ??= await getSupportedBiometricsUseCase.invoke();
      final state = await isBiometricLoginEnabledUseCase.invoke();
      emit(BiometricSettingsLoaded(biometricLoginEnabled: state));
    } catch (ex) {
      Fimber.e('Failed to get state', ex: ex);
      emit(const BiometricSettingsError());
    }
  }

  Future<void> _onBiometricUnlockSettingToggled(event, emit) async {
    final localState = state;
    if (localState is BiometricSettingsLoaded) {
      if (localState.biometricLoginEnabled) {
        await _disableBiometrics(emit, localState);
      } else {
        await _enableBiometrics(emit, localState);
      }
    } else {
      Fimber.e("Can't toggle biometric unlock while in state: $state");
    }
  }

  Future<void> _disableBiometrics(emit, BiometricSettingsLoaded currentState) async {
    try {
      emit(const BiometricSettingsLoaded(biometricLoginEnabled: false));
      await setBiometricsUseCase.invoke(enable: false, authenticateBeforeEnabling: false);
    } catch (ex) {
      Fimber.e('Failed to update state', ex: ex);
      // Revert state changes
      emit(currentState);
    }
  }

  Future<void> _enableBiometrics(emit, BiometricSettingsLoaded currentState) async {
    try {
      // Eagerly update the UI
      emit(const BiometricSettingsLoaded(biometricLoginEnabled: true));

      // Perform biometric authentication
      final result = await requestBiometricsUsecaseImpl.invoke();
      switch (result) {
        case RequestBiometricsResult.success:
          emit(const BiometricSettingsConfirmPin()); // Request PIN to confirm
        case RequestBiometricsResult.failure:
          emit(currentState);
        case RequestBiometricsResult.setupRequired:
          emit(const BiometricSettingsSetupRequired());
          emit(currentState);
      }
    } catch (ex) {
      Fimber.e('Failed to update state', ex: ex);
      emit(currentState);
    }
  }

  Future<void> _onBiometricUnlockEnabled(event, emit) async {
    try {
      final availability = await getAvailableBiometricsUseCase.invoke();
      if (availability == Biometrics.none) {
        throw UnsupportedError('Biometrics can only be enabled when they are available');
      }
      await setBiometricsUseCase.invoke(enable: true, authenticateBeforeEnabling: false);
      emit(const BiometricSettingsLoaded(biometricLoginEnabled: true));
    } catch (ex) {
      Fimber.e('Failed to get state', ex: ex);
      emit(const BiometricSettingsError());
    }
  }
}
