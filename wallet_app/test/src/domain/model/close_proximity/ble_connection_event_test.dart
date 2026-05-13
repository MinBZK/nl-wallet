import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/close_proximity/ble_connection_event.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

void main() {
  group('equals', () {
    test('BleAdvertising', () {
      final a = const BleAdvertising('a');
      final a2 = const BleAdvertising('a');
      final b = const BleAdvertising('b');

      expect(a, isNot(equals(b)));
      expect(a, equals(a2));
    });

    test('BleConnected', () {
      final a = const BleConnected();
      final a2 = const BleConnected();
      expect(a, equals(a2));
    });

    test('BleDeviceRequestReceived', () {
      final a = const BleDeviceRequestReceived();
      final a2 = const BleDeviceRequestReceived();
      expect(a, equals(a2));
    });

    test('BleDisconnected', () {
      final a = const BleDisconnected();
      final a2 = const BleDisconnected();
      expect(a, equals(a2));
    });

    test('BleError', () {
      const error = CoreGenericError('test');
      final a = const BleError(error);
      final a2 = const BleError(error);
      final b = const BleError(CoreCancelledSessionError('test'));

      expect(a, isNot(equals(b)));
      expect(a, equals(a2));
    });
  });
}
