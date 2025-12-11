import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:mockito/mockito.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/domain/model/permission/permission_check_result.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/qr/bloc/qr_bloc.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockDecodeQrUseCase decodeQrUseCase;
  late MockRequestPermissionUseCase requestPermissionUseCase;

  setUp(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
      const MethodChannel('vibration'),
      (MethodCall methodCall) {
        return null;
      },
    );
    decodeQrUseCase = MockDecodeQrUseCase();
    requestPermissionUseCase = MockRequestPermissionUseCase();
  });

  blocTest(
    'ltc7 ltc16 ltc19 verify initial state',
    build: () => QrBloc(decodeQrUseCase, requestPermissionUseCase),
    verify: (bloc) {
      expect(bloc.state, QrScanInitial());
    },
  );

  blocTest(
    'ltc7 ltc16 ltc19 verify permission state when permission is not permanentlyDenied',
    build: () => QrBloc(decodeQrUseCase, requestPermissionUseCase),
    act: (bloc) => bloc.add(const QrScanCheckPermission()),
    setUp: () {
      when(
        requestPermissionUseCase.invoke(Permission.camera),
      ).thenAnswer((_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false));
    },
    expect: () => [const QrScanNoPermission(permanentlyDenied: false)],
  );

  blocTest(
    'ltc7 ltc16 ltc19 verify permission state when permission is permanentlyDenied',
    build: () => QrBloc(decodeQrUseCase, requestPermissionUseCase),
    act: (bloc) => bloc.add(const QrScanCheckPermission()),
    setUp: () {
      when(
        requestPermissionUseCase.invoke(Permission.camera),
      ).thenAnswer((_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: true));
    },
    expect: () => [const QrScanNoPermission(permanentlyDenied: true)],
  );

  blocTest(
    'ltc7 ltc16 ltc19 verify scanner moves to scanning when permission is granted',
    build: () => QrBloc(decodeQrUseCase, requestPermissionUseCase),
    act: (bloc) => bloc.add(const QrScanCheckPermission()),
    setUp: () {
      when(
        requestPermissionUseCase.invoke(Permission.camera),
      ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
    },
    expect: () => [QrScanScanning()],
  );

  blocTest(
    'ltc7 ltc16 ltc19 verify scanner moves to scan failed when scanner throws',
    build: () => QrBloc(decodeQrUseCase, requestPermissionUseCase),
    act: (bloc) => bloc
      ..add(const QrScanCheckPermission())
      ..add(const QrScanCodeDetected(Barcode())),
    setUp: () {
      when(
        requestPermissionUseCase.invoke(Permission.camera),
      ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      when(
        decodeQrUseCase.invoke(any),
      ).thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test')));
    },
    wait: const Duration(milliseconds: 100),
    expect: () => [QrScanScanning(), const QrScanLoading(), QrScanFailure()],
  );

  blocTest(
    'ltc7 ltc16 ltc19 verify scanner moves to scan failed when scanner returns an error',
    build: () => QrBloc(decodeQrUseCase, requestPermissionUseCase),
    act: (bloc) => bloc
      ..add(const QrScanCheckPermission())
      ..add(const QrScanCodeDetected(Barcode())),
    setUp: () {
      when(
        requestPermissionUseCase.invoke(Permission.camera),
      ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      when(
        decodeQrUseCase.invoke(any),
      ).thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test')));
    },
    wait: const Duration(milliseconds: 100),
    expect: () => [QrScanScanning(), const QrScanLoading(), QrScanFailure()],
  );

  blocTest(
    'ltc7 ltc16 ltc19 verify scanner moves to scan success when scanner returns a valid result',
    build: () => QrBloc(decodeQrUseCase, requestPermissionUseCase),
    act: (bloc) => bloc
      ..add(const QrScanCheckPermission())
      ..add(const QrScanCodeDetected(Barcode())),
    setUp: () {
      when(
        requestPermissionUseCase.invoke(Permission.camera),
      ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      when(
        decodeQrUseCase.invoke(any),
      ).thenAnswer((_) async => const Result.success(GenericNavigationRequest('/destination')));
    },
    wait: const Duration(milliseconds: 100),
    expect: () => [
      QrScanScanning(),
      const QrScanLoading(),
      const QrScanSuccess(GenericNavigationRequest('/destination')),
    ],
  );

  blocTest(
    'ltc7 ltc16 ltc19 triggering multiple scans only should only result in one decode attempt (i.e. process one barcode at a time)',
    build: () => QrBloc(decodeQrUseCase, requestPermissionUseCase),
    act: (bloc) => bloc
      ..add(const QrScanCheckPermission())
      ..add(const QrScanCodeDetected(Barcode(rawValue: 'a')))
      ..add(const QrScanCodeDetected(Barcode(rawValue: 'b')))
      ..add(const QrScanCodeDetected(Barcode(rawValue: 'c'))),
    setUp: () {
      when(
        requestPermissionUseCase.invoke(Permission.camera),
      ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      when(
        decodeQrUseCase.invoke(any),
      ).thenAnswer((_) async => const Result.success(GenericNavigationRequest('/destination')));
    },
    verify: (bloc) => verify(decodeQrUseCase.invoke(any)).called(1),
    wait: const Duration(milliseconds: 100),
    expect: () => [
      QrScanScanning(),
      const QrScanLoading(),
      const QrScanSuccess(GenericNavigationRequest('/destination')),
    ],
  );

  blocTest(
    'ltc7 ltc16 ltc19 resetting the scanner should allow the next uri to be decoded',
    build: () => QrBloc(decodeQrUseCase, requestPermissionUseCase),
    act: (bloc) async {
      bloc
        ..add(const QrScanCheckPermission())
        ..add(const QrScanCodeDetected(Barcode(rawValue: 'a')));
      // Wait for initial Scan to be fully processed
      await Future.delayed(const Duration(milliseconds: 200));
      bloc
        ..add(const QrScanReset())
        ..add(const QrScanCodeDetected(Barcode(rawValue: 'b')));
    },
    setUp: () {
      when(
        requestPermissionUseCase.invoke(Permission.camera),
      ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      when(decodeQrUseCase.invoke(any)).thenAnswer((invocation) async {
        final Barcode barcode = invocation.positionalArguments.first;
        return Result.success(GenericNavigationRequest(barcode.rawValue!));
      });
    },
    verify: (bloc) => verify(decodeQrUseCase.invoke(any)).called(2),
    wait: const Duration(milliseconds: 100),
    expect: () => [
      QrScanScanning(),
      const QrScanLoading(),
      const QrScanSuccess(GenericNavigationRequest('a')),
      QrScanInitial(),
      const QrScanLoading(),
      QrScanScanning(),
      const QrScanSuccess(GenericNavigationRequest('b')),
    ],
  );
}
