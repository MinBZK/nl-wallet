import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/domain/model/configuration/maintenance_window.dart';
import 'package:wallet/src/util/mapper/configuration/flutter_app_configuration_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../mocks/wallet_mocks.dart';

void main() {
  late Mapper<(String, String)?, MaintenanceWindow?> mockMaintenanceWindowMapper;

  late Mapper<core.FlutterConfiguration, FlutterAppConfiguration> mapper;

  setUp(() {
    mockMaintenanceWindowMapper = MockMapper();

    mapper = FlutterAppConfigurationMapper(
      mockMaintenanceWindowMapper,
    );
  });

  group('map', () {
    test('maps all fields from FlutterConfiguration to FlutterAppConfiguration', () {
      const input = core.FlutterConfiguration(
        inactiveLockTimeout: 300,
        inactiveWarningTimeout: 60,
        backgroundLockTimeout: 600,
        staticAssetsBaseUrl: 'https://example.com/',
        pidAttestationTypes: ['urn:eudi:pid:nl:1'],
        maintenanceWindow: null,
        version: '1.0.0',
        environment: 'production',
      );

      const expectedOutput = FlutterAppConfiguration(
        idleLockTimeout: Duration(seconds: 300),
        idleWarningTimeout: Duration(seconds: 60),
        backgroundLockTimeout: Duration(seconds: 600),
        staticAssetsBaseUrl: 'https://example.com/',
        pidAttestationTypes: ['urn:eudi:pid:nl:1'],
        maintenanceWindow: null,
        version: '1.0.0',
        environment: 'production',
      );

      expect(mapper.map(input), expectedOutput);

      verify(mockMaintenanceWindowMapper.map(null)).called(1);
    });
  });
}
