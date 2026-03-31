import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/close_proximity/ble_connection_event.dart';
import 'package:wallet/src/util/mapper/close_proximity/close_proximity_disclosure_update_mapper.dart';
import 'package:wallet_core/core.dart' as core;

void main() {
  late CloseProximityDisclosureUpdateMapper mapper;

  setUp(() {
    mapper = CloseProximityDisclosureUpdateMapper();
  });

  group('CloseProximityDisclosureUpdateMapper', () {
    test('maps Connecting to BleConnecting', () {
      final result = mapper.map(core.CloseProximityDisclosureFlutterUpdate.Connecting);
      expect(result, isA<BleConnecting>());
    });

    test('maps Connected to BleConnected', () {
      final result = mapper.map(core.CloseProximityDisclosureFlutterUpdate.Connected);
      expect(result, isA<BleConnected>());
    });

    test('maps DeviceRequestReceived to BleDeviceRequestReceived', () {
      final result = mapper.map(core.CloseProximityDisclosureFlutterUpdate.DeviceRequestReceived);
      expect(result, isA<BleDeviceRequestReceived>());
    });

    test('maps Disconnected to BleDisconnected', () {
      final result = mapper.map(core.CloseProximityDisclosureFlutterUpdate.Disconnected);
      expect(result, isA<BleDisconnected>());
    });
  });
}
