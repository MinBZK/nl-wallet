import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/configuration/flutter_app_configuration.dart';
import '../../../domain/model/configuration/maintenance_window.dart';
import '../mapper.dart';

class FlutterAppConfigurationMapper extends Mapper<core.FlutterConfiguration, FlutterAppConfiguration> {
  final Mapper<(String, String)?, MaintenanceWindow?> _maintenanceWindowMapper;

  FlutterAppConfigurationMapper(this._maintenanceWindowMapper);

  @override
  FlutterAppConfiguration map(core.FlutterConfiguration input) {
    return FlutterAppConfiguration(
      idleLockTimeout: Duration(seconds: input.inactiveLockTimeout),
      idleWarningTimeout: Duration(seconds: input.inactiveWarningTimeout),
      backgroundLockTimeout: Duration(seconds: input.backgroundLockTimeout),
      staticAssetsBaseUrl: input.staticAssetsBaseUrl,
      pidAttestationTypes: input.pidAttestationTypes,
      maintenanceWindow: _maintenanceWindowMapper.map(input.maintenanceWindow),
      version: input.version,
      environment: input.environment,
    );
  }
}
