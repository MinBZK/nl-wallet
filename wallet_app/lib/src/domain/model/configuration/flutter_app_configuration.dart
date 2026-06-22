import 'package:freezed_annotation/freezed_annotation.dart';

import '../pid/pid_attestation.dart';
import 'maintenance_window.dart';

part 'flutter_app_configuration.freezed.dart';

@freezed
abstract class FlutterAppConfiguration with _$FlutterAppConfiguration {
  const factory FlutterAppConfiguration({
    required Duration idleLockTimeout,
    required Duration idleWarningTimeout,
    required Duration backgroundLockTimeout,
    required String staticAssetsBaseUrl,
    required List<PidAttestation> pidAttestations,
    required MaintenanceWindow? maintenanceWindow,
    required String version,
    required String environment,
  }) = _FlutterAppConfiguration;

  const FlutterAppConfiguration._();

  String get versionAndEnvironment => '$version ($environment)';

  Set<String> get pidAttestationTypes => pidAttestations.map((it) => it.attestationType).toSet();
}
