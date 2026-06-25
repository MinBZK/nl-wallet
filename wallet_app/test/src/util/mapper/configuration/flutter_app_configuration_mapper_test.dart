import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/domain/model/configuration/maintenance_window.dart';
import 'package:wallet/src/domain/model/pid/pid_attestation.dart';
import 'package:wallet/src/util/mapper/configuration/flutter_app_configuration_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../mocks/wallet_mocks.dart';

void main() {
  late Mapper<(String, String)?, MaintenanceWindow?> mockMaintenanceWindowMapper;
  late Mapper<core.PidAttestation, PidAttestation> mockPidAttestationMapper;

  late Mapper<core.FlutterConfiguration, FlutterAppConfiguration> mapper;

  setUp(() {
    mockMaintenanceWindowMapper = MockMapper();
    mockPidAttestationMapper = MockMapper();

    mapper = FlutterAppConfigurationMapper(mockMaintenanceWindowMapper, mockPidAttestationMapper);
  });

  tearDown(() {
    reset(mockMaintenanceWindowMapper);
    reset(mockPidAttestationMapper);
  });

  group('map', () {
    test('maps all fields from FlutterConfiguration to FlutterAppConfiguration', () {
      // Setup mock pid attestation mapper
      const corePidAttestation = core.PidAttestation(format: core.Format.SdJwt, attestationType: 'urn:eudi:pid:nl:1');
      const pidAttestation = PidAttestation(attestationType: 'urn:eudi:pid:nl:1', format: .sdJwt);
      when(mockPidAttestationMapper.mapList([corePidAttestation])).thenReturn([pidAttestation]);

      const input = core.FlutterConfiguration(
        inactiveLockTimeout: 300,
        inactiveWarningTimeout: 60,
        backgroundLockTimeout: 600,
        staticAssetsBaseUrl: 'https://example.com/',
        pidAttestations: [corePidAttestation],
        maintenanceWindow: null,
        version: '1.0.0',
        environment: 'production',
      );

      const expectedOutput = FlutterAppConfiguration(
        idleLockTimeout: Duration(seconds: 300),
        idleWarningTimeout: Duration(seconds: 60),
        backgroundLockTimeout: Duration(seconds: 600),
        staticAssetsBaseUrl: 'https://example.com/',
        pidAttestations: [pidAttestation],
        maintenanceWindow: null,
        version: '1.0.0',
        environment: 'production',
      );

      expect(mapper.map(input), expectedOutput);

      verify(mockMaintenanceWindowMapper.map(null)).called(1);
    });
  });
}
