import 'package:clock/clock.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'maintenance_window.freezed.dart';

@freezed
abstract class MaintenanceWindow with _$MaintenanceWindow {
  const factory MaintenanceWindow({
    required DateTime startDateTime,
    required DateTime endDateTime,
  }) = _MaintenanceWindow;

  const MaintenanceWindow._();

  /// Check if the current time falls within the maintenance window
  /// Start time is inclusive, end time is exclusive (standard for time ranges)
  bool get isCurrentlyInMaintenance {
    final now = clock.now().toUtc();
    final start = startDateTime.toUtc();
    final end = endDateTime.toUtc();
    return !now.isBefore(start) && now.isBefore(end);
  }

  /// Check if the maintenance window is valid (start before end)
  bool get isValid => startDateTime.isBefore(endDateTime);
}
