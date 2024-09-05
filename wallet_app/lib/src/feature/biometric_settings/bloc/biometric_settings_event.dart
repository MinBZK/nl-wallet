part of 'biometric_settings_bloc.dart';

abstract class BiometricSettingsEvent extends Equatable {
  const BiometricSettingsEvent();

  @override
  List<Object?> get props => [];
}

/// Request BLoC to refresh data
class BiometricLoadTriggered extends BiometricSettingsEvent {
  const BiometricLoadTriggered();
}

/// Notify BLoC about user toggling the biometric switch
class BiometricUnlockToggled extends BiometricSettingsEvent {
  const BiometricUnlockToggled();
}

/// Notify BLoC about successful pin confirmation (to enable biometric unlock)
class BiometricUnlockEnabledWithPin extends BiometricSettingsEvent {
  const BiometricUnlockEnabledWithPin();
}
