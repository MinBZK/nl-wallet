part of 'developer_settings_bloc.dart';

sealed class DeveloperSettingsEvent extends Equatable {
  const DeveloperSettingsEvent();

  @override
  List<Object?> get props => [];
}

class DeveloperSettingsLoadTriggered extends DeveloperSettingsEvent {
  const DeveloperSettingsLoadTriggered();
}

class DeveloperSettingsWrprcStrictModeToggled extends DeveloperSettingsEvent {
  final bool enabled;

  const DeveloperSettingsWrprcStrictModeToggled({required this.enabled});

  @override
  List<Object?> get props => [enabled];
}
