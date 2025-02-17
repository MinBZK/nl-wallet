import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:mockito/mockito.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/usecase/permission/check_has_permission_usecase.dart';
import 'package:wallet/src/feature/qr/bloc/qr_bloc.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockDecodeQrUseCase decodeQrUseCase;
  late MockCheckHasPermissionUseCase checkHasPermissionUseCase;

  setUp(() {
    provideDummy<Result<NavigationRequest>>(const Result.success(GenericNavigationRequest('')));
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(const MethodChannel('vibration'), (MethodCall methodCall) {
      return null;
    });
    decodeQrUseCase = MockDecodeQrUseCase();
    checkHasPermissionUseCase = MockCheckHasPermissionUseCase();
  });

  blocTest(
    'verify initial state',
    build: () => QrBloc(decodeQrUseCase, checkHasPermissionUseCase),
    verify: (bloc) {
      expect(bloc.state, QrScanInitial());
    },
  );

  blocTest(
    'verify permission state when permission is not permanentlyDenied',
    build: () => QrBloc(decodeQrUseCase, checkHasPermissionUseCase),
    act: (bloc) => bloc.add(const QrScanCheckPermission()),
    setUp: () {
      when(checkHasPermissionUseCase.invoke(Permission.camera))
          .thenAnswer((_) async => PermissionCheckResult(isGranted: false, isPermanentlyDenied: false));
    },
    expect: () => [const QrScanNoPermission(permanentlyDenied: false)],
  );

  blocTest(
    'verify permission state when permission is permanentlyDenied',
    build: () => QrBloc(decodeQrUseCase, checkHasPermissionUseCase),
    act: (bloc) => bloc.add(const QrScanCheckPermission()),
    setUp: () {
      when(checkHasPermissionUseCase.invoke(Permission.camera))
          .thenAnswer((_) async => PermissionCheckResult(isGranted: false, isPermanentlyDenied: true));
    },
    expect: () => [const QrScanNoPermission(permanentlyDenied: true)],
  );

  blocTest(
    'verify scanner moves to scanning when permission is granted',
    build: () => QrBloc(decodeQrUseCase, checkHasPermissionUseCase),
    act: (bloc) => bloc.add(const QrScanCheckPermission()),
    setUp: () {
      when(checkHasPermissionUseCase.invoke(Permission.camera))
          .thenAnswer((_) async => PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
    },
    expect: () => [QrScanScanning()],
  );

  blocTest(
    'verify scanner moves to scan failed when scanner throws',
    build: () => QrBloc(decodeQrUseCase, checkHasPermissionUseCase),
    act: (bloc) => bloc
      ..add(const QrScanCheckPermission())
      ..add(const QrScanCodeDetected(Barcode())),
    setUp: () {
      when(checkHasPermissionUseCase.invoke(Permission.camera))
          .thenAnswer((_) async => PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      when(decodeQrUseCase.invoke(any))
          .thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test')));
    },
    wait: const Duration(milliseconds: 100),
    expect: () => [QrScanScanning(), const QrScanLoading(), QrScanFailure()],
  );

  blocTest(
    'verify scanner moves to scan failed when scanner returns an error',
    build: () => QrBloc(decodeQrUseCase, checkHasPermissionUseCase),
    act: (bloc) => bloc
      ..add(const QrScanCheckPermission())
      ..add(const QrScanCodeDetected(Barcode())),
    setUp: () {
      when(checkHasPermissionUseCase.invoke(Permission.camera))
          .thenAnswer((_) async => PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      when(decodeQrUseCase.invoke(any))
          .thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test')));
    },
    wait: const Duration(milliseconds: 100),
    expect: () => [QrScanScanning(), const QrScanLoading(), QrScanFailure()],
  );

  blocTest(
    'verify scanner moves to scan success when scanner returns a valid result',
    build: () => QrBloc(decodeQrUseCase, checkHasPermissionUseCase),
    act: (bloc) => bloc
      ..add(const QrScanCheckPermission())
      ..add(const QrScanCodeDetected(Barcode())),
    setUp: () {
      when(checkHasPermissionUseCase.invoke(Permission.camera))
          .thenAnswer((_) async => PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      when(decodeQrUseCase.invoke(any))
          .thenAnswer((_) async => const Result.success(GenericNavigationRequest('/destination')));
    },
    wait: const Duration(milliseconds: 100),
    expect: () => [
      QrScanScanning(),
      const QrScanLoading(),
      const QrScanSuccess(GenericNavigationRequest('/destination')),
    ],
  );

  blocTest(
    'triggering multiple scans only should only result in one decode attempt (i.e. process one barcode at a time)',
    build: () => QrBloc(decodeQrUseCase, checkHasPermissionUseCase),
    act: (bloc) => bloc
      ..add(const QrScanCheckPermission())
      ..add(const QrScanCodeDetected(Barcode(rawValue: 'a')))
      ..add(const QrScanCodeDetected(Barcode(rawValue: 'b')))
      ..add(const QrScanCodeDetected(Barcode(rawValue: 'c'))),
    setUp: () {
      when(checkHasPermissionUseCase.invoke(Permission.camera))
          .thenAnswer((_) async => PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
      when(decodeQrUseCase.invoke(any))
          .thenAnswer((_) async => const Result.success(GenericNavigationRequest('/destination')));
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
    'resetting the scanner should allow the next uri to be decoded',
    build: () => QrBloc(decodeQrUseCase, checkHasPermissionUseCase),
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
      when(checkHasPermissionUseCase.invoke(Permission.camera))
          .thenAnswer((_) async => PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
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
