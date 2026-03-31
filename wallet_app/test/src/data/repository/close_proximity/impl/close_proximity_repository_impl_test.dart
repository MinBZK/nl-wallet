import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/close_proximity/impl/close_proximity_repository_impl.dart';
import 'package:wallet/src/domain/model/close_proximity/ble_connection_event.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late CloseProximityRepositoryImpl repository;
  late MockTypedWalletCore mockTypedWalletCore;
  late MockMapper<core.CloseProximityDisclosureFlutterUpdate, BleConnectionEvent> mockMapper;

  setUp(() {
    mockTypedWalletCore = MockTypedWalletCore();
    mockMapper = MockMapper();
    repository = CloseProximityRepositoryImpl(mockTypedWalletCore, mockMapper);
  });

  group('CloseProximityRepositoryImpl', () {
    test('startCloseProximityDisclosure calls core and emits events', () async {
      const engagement = 'mdoc:engagement_data';

      // Capture the callback passed to the core
      late FutureOr<void> Function(core.CloseProximityDisclosureFlutterUpdate) capturedCallback;
      when(
        mockTypedWalletCore.startCloseProximityDisclosure(
          callback: anyNamed('callback'),
        ),
      ).thenAnswer((invocation) async {
        capturedCallback = invocation.namedArguments[#callback];
        return engagement;
      });

      final observedEvents = <BleConnectionEvent>[];
      final subscription = repository.observeBleConnectionEvents().listen(observedEvents.add);

      final engagementResult = await repository.startCloseProximityDisclosure();

      expect(engagementResult, engagement);
      verify(mockTypedWalletCore.startCloseProximityDisclosure(callback: anyNamed('callback'))).called(1);

      // Verify initial BleAdvertising event is emitted
      await Future.microtask(() {});
      expect(observedEvents, contains(const BleAdvertising(engagement)));

      // Simulate core update
      const update = core.CloseProximityDisclosureFlutterUpdate.Connecting;
      const mappedEvent = BleConnecting();
      when(mockMapper.map(update)).thenReturn(mappedEvent);

      // Push Connecting into the captured callback
      await capturedCallback(update);

      // Verify (mapped) Connecting event is emitted
      expect(observedEvents, contains(mappedEvent));

      await subscription.cancel();
    });

    test('observeBleConnectionEvents returns a stream of events', () async {
      final stream = repository.observeBleConnectionEvents();
      expect(stream, isA<Stream<BleConnectionEvent>>());
    });

    test('multiple core updates are correctly mapped and emitted', () async {
      const engagement = 'mdoc:engagement_data';
      late FutureOr<void> Function(core.CloseProximityDisclosureFlutterUpdate) capturedCallback;

      when(
        mockTypedWalletCore.startCloseProximityDisclosure(
          callback: anyNamed('callback'),
        ),
      ).thenAnswer((invocation) async {
        capturedCallback = invocation.namedArguments[#callback];
        return engagement;
      });

      final observedEvents = <BleConnectionEvent>[];
      final subscription = repository.observeBleConnectionEvents().listen(observedEvents.add);

      await repository.startCloseProximityDisclosure();

      // Define updates and their expected mappings
      final updates = [
        (core.CloseProximityDisclosureFlutterUpdate.Connecting, const BleConnecting()),
        (core.CloseProximityDisclosureFlutterUpdate.Connected, const BleConnected()),
        (core.CloseProximityDisclosureFlutterUpdate.DeviceRequestReceived, const BleDeviceRequestReceived()),
        (core.CloseProximityDisclosureFlutterUpdate.Disconnected, const BleDisconnected()),
      ];

      // Publish all updates into the captured callback
      for (final (update, expectedEvent) in updates) {
        when(mockMapper.map(update)).thenReturn(expectedEvent);
        await capturedCallback(update);
      }

      // Verify these updates are emitted in order (with initial advertising event)
      await Future.microtask(() {});
      expect(observedEvents, [
        const BleAdvertising(engagement),
        const BleConnecting(),
        const BleConnected(),
        const BleDeviceRequestReceived(),
        const BleDisconnected(),
      ]);

      await subscription.cancel();
    });

    test('startCloseProximityDisclosure propagates error from core', () async {
      when(
        mockTypedWalletCore.startCloseProximityDisclosure(
          callback: anyNamed('callback'),
        ),
      ).thenThrow(Exception('Core error'));

      expect(() => repository.startCloseProximityDisclosure(), throwsException);
    });
  });
}
