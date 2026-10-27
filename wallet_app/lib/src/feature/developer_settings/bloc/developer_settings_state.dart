part of 'developer_settings_bloc.dart';

class DeveloperSettingsState extends Equatable {
  /// The last successfully loaded or saved value, or null until loading succeeds.
  final bool? wrprcStrictMode;
  final bool isBusy;
  final bool hasError;

  const DeveloperSettingsState({
    this.wrprcStrictMode,
    this.isBusy = false,
    this.hasError = false,
  });

  bool get canToggleWrprcStrictMode => wrprcStrictMode != null && !isBusy;

  @override
  List<Object?> get props => [wrprcStrictMode, isBusy, hasError];
}
