import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/configuration/flutter_app_configuration.dart';
import '../../../domain/model/configuration/maintenance_window.dart';
import '../../../domain/model/pid/pid_attestation.dart';
import '../mapper.dart';

class FlutterAppConfigurationMapper extends Mapper<core.FlutterConfiguration, FlutterAppConfiguration> {
  final Mapper<(String, String)?, MaintenanceWindow?> _maintenanceWindowMapper;
  final Mapper<core.PidAttestation, PidAttestation> _pidAttestationMapper;

  FlutterAppConfigurationMapper(this._maintenanceWindowMapper, this._pidAttestationMapper);

  @override
  FlutterAppConfiguration map(core.FlutterConfiguration input) {
    return FlutterAppConfiguration(
      idleLockTimeout: Duration(seconds: input.inactiveLockTimeout),
      idleWarningTimeout: Duration(seconds: input.inactiveWarningTimeout),
      backgroundLockTimeout: Duration(seconds: input.backgroundLockTimeout),
      staticAssetsBaseUrl: input.staticAssetsBaseUrl,
      pidAttestations: _pidAttestationMapper.mapList(input.pidAttestations),
      maintenanceWindow: _maintenanceWindowMapper.map(input.maintenanceWindow),
      version: input.version,
      environment: input.environment,
    );
  }
}
