import 'package:freezed_annotation/freezed_annotation.dart';

import 'maintenance_window.dart';

part 'flutter_app_configuration.freezed.dart';

@freezed
abstract class FlutterAppConfiguration with _$FlutterAppConfiguration {
  const factory FlutterAppConfiguration({
    required Duration idleLockTimeout,
    required Duration idleWarningTimeout,
    required Duration backgroundLockTimeout,
    required String staticAssetsBaseUrl,
    required List<String> pidAttestationTypes,
    required MaintenanceWindow? maintenanceWindow,
    required String version,
    required String environment,
  }) = _FlutterAppConfiguration;

  const FlutterAppConfiguration._();

  String get versionAndEnvironment => '$version ($environment)';
}
