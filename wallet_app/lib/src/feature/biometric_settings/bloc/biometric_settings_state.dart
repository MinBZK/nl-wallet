part of 'biometric_settings_bloc.dart';

sealed class BiometricSettingsState extends Equatable {
  const BiometricSettingsState();
}

class BiometricSettingsInitial extends BiometricSettingsState {
  @override
  List<Object> get props => [];
}

class BiometricSettingsLoading extends BiometricSettingsState {
  final bool? biometricLoginEnabled;

  const BiometricSettingsLoading({this.biometricLoginEnabled});

  @override
  List<Object?> get props => [biometricLoginEnabled];
}

class BiometricSettingsError extends BiometricSettingsState {
  const BiometricSettingsError();

  @override
  List<Object> get props => [];
}

class BiometricSettingsConfirmPin extends BiometricSettingsState {
  const BiometricSettingsConfirmPin();

  @override
  List<Object> get props => [];
}

class BiometricSettingsSetupRequired extends BiometricSettingsState {
  const BiometricSettingsSetupRequired();

  @override
  List<Object> get props => [];
}

class BiometricSettingsLoaded extends BiometricSettingsState {
  final bool biometricLoginEnabled;

  const BiometricSettingsLoaded({required this.biometricLoginEnabled});

  BiometricSettingsLoaded toggled() => BiometricSettingsLoaded(biometricLoginEnabled: !biometricLoginEnabled);

  @override
  List<Object> get props => [biometricLoginEnabled];
}
