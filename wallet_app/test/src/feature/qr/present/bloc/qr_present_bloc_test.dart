import 'package:bloc_test/bloc_test.dart';
import 'package:bluetooth/bluetooth.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/app_lifecycle_service.dart';
import 'package:wallet/src/domain/model/close_proximity/ble_connection_event.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/qr/present/bloc/qr_present_bloc.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockStartCloseProximityDisclosureUseCase mockStartQrEngagementUseCase;
  late MockCancelDisclosureUseCase mockCancelDisclosureUseCase;
  late MockObserveCloseProximityConnectionUseCase mockObserveCloseProximityConnectionUseCase;
  late AppLifecycleService mockAppLifecycleService;
  late QrPresentBloc bloc;
  late Bluetooth bluetooth;

  setUp(() {
    mockStartQrEngagementUseCase = MockStartCloseProximityDisclosureUseCase();
    mockObserveCloseProximityConnectionUseCase = MockObserveCloseProximityConnectionUseCase();
    mockCancelDisclosureUseCase = MockCancelDisclosureUseCase();
    bluetooth = MockBluetooth();
    mockAppLifecycleService = AppLifecycleService();
    bloc = QrPresentBloc(
      mockStartQrEngagementUseCase,
      mockObserveCloseProximityConnectionUseCase,
      mockCancelDisclosureUseCase,
      bluetooth,
      mockAppLifecycleService,
    );
  });

  group('QrPresentBloc', () {
    test('initial state is QrPresentInitial', () {
      expect(bloc.state, const QrPresentInitial());
    });

    blocTest<QrPresentBloc, QrPresentState>(
      'emits [QrPresentInitial, QrPresentServerStarted] when QrPresentStartRequested is added and usecase succeeds',
      build: () => bloc,
      setUp: () {
        when(bluetooth.isEnabled()).thenAnswer((_) async => true);
        when(mockStartQrEngagementUseCase.invoke()).thenAnswer((_) async => const Result.success('qr_content'));
      },
      act: (bloc) => bloc.add(const QrPresentStartRequested()),
      expect: () => [
        const QrPresentInitial(),
        const QrPresentServerStarted('qr_content'),
      ],
      verify: (_) {
        verify(mockStartQrEngagementUseCase.invoke()).called(1);
      },
    );

    blocTest<QrPresentBloc, QrPresentState>(
      'emits [QrPresentInitial, QrPresentBluetoothDisabled] when bluetooth system setting is disabled',
      build: () => bloc,
      setUp: () {
        when(bluetooth.isEnabled()).thenAnswer((_) async => false);
      },
      act: (bloc) => bloc.add(const QrPresentStartRequested()),
      expect: () => [
        const QrPresentInitial(),
        const QrPresentBluetoothDisabled(),
      ],
    );

    blocTest<QrPresentBloc, QrPresentState>(
      'emits [QrPresentInitial, QrPresentError] when QrPresentStartRequested is added and usecase fails',
      build: () => bloc,
      setUp: () {
        when(bluetooth.isEnabled()).thenAnswer((_) async => true);
        when(mockStartQrEngagementUseCase.invoke()).thenAnswer(
          (_) async => const Result.error(GenericError('error', sourceError: 'error')),
        );
      },
      act: (bloc) => bloc.add(const QrPresentStartRequested()),
      expect: () => [
        const QrPresentInitial(),
        const QrPresentError(GenericError('error', sourceError: 'error')),
      ],
      verify: (_) {
        verify(mockStartQrEngagementUseCase.invoke()).called(1);
      },
    );

    test('Cancel disclosure on stop request', () async {
      bloc.add(const QrPresentStopRequested());
      await Future.microtask(() {}); // Process event

      verify(mockCancelDisclosureUseCase.invoke()).called(1);
    });

    test('Cancel disclosure on missing permission and emit a generic error', () async {
      bloc.add(const QrPresentPermissionDenied());
      await Future.microtask(() {}); // Process event

      verify(mockCancelDisclosureUseCase.invoke()).called(1);

      expect(bloc.state, isA<QrPresentError>().having((it) => it.error, 'exposes generic error', isA<GenericError>()));
    });

    test('Cancel disclosure on bloc close', () async {
      await bloc.close();

      verify(mockCancelDisclosureUseCase.invoke()).called(1);
    });

    test('Do not cancel disclosure on bloc close when state is connected and device request received', () async {
      bloc.add(const QrPresentEventReceived(BleDeviceRequestReceived()));
      await Future.microtask(() {}); // Process event

      expect(bloc.state, const QrPresentConnected(deviceRequestReceived: true));

      await bloc.close();

      verifyNever(mockCancelDisclosureUseCase.invoke());
    });

    group('Lifecycle', () {
      blocTest<QrPresentBloc, QrPresentState>(
        'adds QrPresentStopRequested when resumed and bluetooth is disabled while in QrPresentServerStarted',
        build: () => bloc,
        seed: () => const QrPresentServerStarted('qr_content'),
        setUp: () {
          when(bluetooth.isEnabled()).thenAnswer((_) async => false);
          when(mockCancelDisclosureUseCase.invoke()).thenAnswer((_) async => const Result.success(null));
        },
        act: (bloc) => mockAppLifecycleService.notifyStateChanged(AppLifecycleState.resumed),
        expect: () => [const QrPresentConnectionFailed()],
        verify: (_) => verify(mockCancelDisclosureUseCase.invoke()).called(2),
      );

      blocTest<QrPresentBloc, QrPresentState>(
        'adds QrPresentStartRequested when resumed and bluetooth is enabled while in QrPresentBluetoothDisabled',
        build: () => bloc,
        seed: () => const QrPresentBluetoothDisabled(),
        setUp: () {
          when(bluetooth.isEnabled()).thenAnswer((_) async => true);
          when(mockStartQrEngagementUseCase.invoke()).thenAnswer((_) async => const Result.success('qr_content'));
        },
        act: (bloc) => mockAppLifecycleService.notifyStateChanged(AppLifecycleState.resumed),
        expect: () => [
          const QrPresentInitial(),
          const QrPresentServerStarted('qr_content'),
        ],
        verify: (_) => verify(mockStartQrEngagementUseCase.invoke()).called(1),
      );

      blocTest<QrPresentBloc, QrPresentState>(
        'does nothing when resumed and bluetooth is still enabled while in QrPresentServerStarted',
        build: () => bloc,
        seed: () => const QrPresentServerStarted('qr_content'),
        setUp: () => when(bluetooth.isEnabled()).thenAnswer((_) async => true),
        act: (bloc) => mockAppLifecycleService.notifyStateChanged(AppLifecycleState.resumed),
        expect: () => [],
      );

      blocTest<QrPresentBloc, QrPresentState>(
        'does nothing when resumed and bluetooth is still disabled while in QrPresentBluetoothDisabled',
        build: () => bloc,
        seed: () => const QrPresentBluetoothDisabled(),
        setUp: () => when(bluetooth.isEnabled()).thenAnswer((_) async => false),
        act: (bloc) => mockAppLifecycleService.notifyStateChanged(AppLifecycleState.resumed),
        expect: () => [],
      );
    });
  });
}
