import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/close_proximity/ble_connection_event.dart';
import 'package:wallet/src/util/mapper/close_proximity/close_proximity_disclosure_update_mapper.dart';
import 'package:wallet/src/wallet_core/error/core_error_mapper.dart';
import 'package:wallet_core/core.dart' as core;

void main() {
  late CloseProximityDisclosureUpdateMapper mapper;

  setUp(() {
    mapper = CloseProximityDisclosureUpdateMapper(CoreErrorMapper());
  });

  group('CloseProximityDisclosureUpdateMapper', () {
    test('maps Connecting to BleConnecting', () {
      final result = mapper.map(const core.CloseProximityDisclosureFlutterUpdate.connecting());
      expect(result, isA<BleConnecting>());
    });

    test('maps Connected to BleConnected', () {
      final result = mapper.map(const core.CloseProximityDisclosureFlutterUpdate.connected());
      expect(result, isA<BleConnected>());
    });

    test('maps DeviceRequestReceived to BleDeviceRequestReceived', () {
      final result = mapper.map(const core.CloseProximityDisclosureFlutterUpdate.deviceRequestReceived());
      expect(result, isA<BleDeviceRequestReceived>());
    });

    test('maps Disconnected to BleDisconnected', () {
      final result = mapper.map(const core.CloseProximityDisclosureFlutterUpdate.disconnected());
      expect(result, isA<BleDisconnected>());
    });

    test('maps Disconnected to BleDisconnected', () {
      final result = mapper.map(
        const core.CloseProximityDisclosureFlutterUpdate.errored(
          error: '{"type":"Generic", "description":"test"}',
        ),
      );
      expect(result, isA<BleError>());
    });
  });
}
