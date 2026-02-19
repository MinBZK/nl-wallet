import 'package:freezed_annotation/freezed_annotation.dart';

import 'maintenance_window.dart';

part 'maintenance_state.freezed.dart';

/// Represents the current maintenance state of the application.
/// This sealed class is designed to work efficiently with distinct() in streams,
/// as the equality comparison will consider both the maintenance window data
/// and whether the app is currently in maintenance mode.
@freezed
sealed class MaintenanceState with _$MaintenanceState {
  /// The app is currently in maintenance mode.
  const factory MaintenanceState.inMaintenance(MaintenanceWindow window) = InMaintenance;

  /// The app is not in maintenance mode.
  const factory MaintenanceState.noMaintenance() = NoMaintenance;
}
