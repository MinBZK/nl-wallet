import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/biometrics/impl/set_biometrics_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/biometrics/set_biometrics_usecase.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockLocalAuthentication localAuthentication;
  late MockActiveLocaleProvider localeProvider;
  late MockBiometricRepository biometricRepository;
  late SetBiometricsUseCase setBiometricsUseCase;

  setUp(() {
    localAuthentication = MockLocalAuthentication();
    localeProvider = MockActiveLocaleProvider();
    when(localeProvider.activeLocale).thenReturn(const Locale('en'));
    biometricRepository = MockBiometricRepository();
    setBiometricsUseCase = SetBiometricsUseCaseImpl(localAuthentication, localeProvider, biometricRepository);
  });

  test('Verify device is checked for compatibility when enabling', () async {
    when(localAuthentication.isDeviceSupported()).thenAnswer((_) async => true);
    await setBiometricsUseCase.invoke(enable: true, authenticateBeforeEnabling: false);
    verify(localAuthentication.isDeviceSupported()).called(1);
  });

  test('Verify authentication is requested when authenticateBeforeEnabling is true', () async {
    when(localAuthentication.isDeviceSupported()).thenAnswer((_) async => true);
    when(
      localAuthentication.authenticate(
        localizedReason: anyNamed('localizedReason'),
        options: anyNamed('options'),
      ),
    ).thenAnswer((_) async => true);
    await setBiometricsUseCase.invoke(enable: true, authenticateBeforeEnabling: true);
    verify(
      localAuthentication.authenticate(
        localizedReason: anyNamed('localizedReason'),
        options: anyNamed('options'),
      ),
    ).called(1);
  });

  test('Verify authentication is NOT requested when authenticateBeforeEnabling is false', () async {
    when(localAuthentication.isDeviceSupported()).thenAnswer((_) async => true);
    when(
      localAuthentication.authenticate(
        localizedReason: anyNamed('localizedReason'),
        options: anyNamed('options'),
      ),
    ).thenAnswer((_) async => true);
    await setBiometricsUseCase.invoke(enable: true, authenticateBeforeEnabling: false);
    verifyNever(
      localAuthentication.authenticate(
        localizedReason: anyNamed('localizedReason'),
        options: anyNamed('options'),
      ),
    );
  });

  test('Verify setting to true fails when device is not supported', () async {
    when(localAuthentication.isDeviceSupported()).thenAnswer((_) async => false);
    await expectLater(
      () async => setBiometricsUseCase.invoke(enable: true, authenticateBeforeEnabling: false),
      throwsA(isA<UnsupportedError>()),
    );
    verifyNever(biometricRepository.enableBiometricLogin());
  });

  test('Verify setting to true fails when authentication fails', () async {
    when(localAuthentication.isDeviceSupported()).thenAnswer((_) async => true);
    when(
      localAuthentication.authenticate(
        localizedReason: anyNamed('localizedReason'),
        options: anyNamed('options'),
      ),
    ).thenAnswer((_) async => false);
    await expectLater(
      () async => setBiometricsUseCase.invoke(enable: true, authenticateBeforeEnabling: true),
      throwsA(isA<Exception>()),
    );
    verifyNever(biometricRepository.enableBiometricLogin());
  });

  test('Verify setting to false succeeds even when device is not supported and auth is requested and fails', () async {
    when(localAuthentication.isDeviceSupported()).thenAnswer((_) async => false);
    when(
      localAuthentication.authenticate(
        localizedReason: anyNamed('localizedReason'),
        options: anyNamed('options'),
      ),
    ).thenAnswer((_) async => false);
    await setBiometricsUseCase.invoke(enable: false, authenticateBeforeEnabling: true);
    verify(biometricRepository.disableBiometricLogin()).called(1);
  });

  test('Verify setting to true succeeds when auth is not requested', () async {
    when(localAuthentication.isDeviceSupported()).thenAnswer((_) async => true);
    await setBiometricsUseCase.invoke(enable: true, authenticateBeforeEnabling: false);
    verifyNever(
      localAuthentication.authenticate(
        localizedReason: anyNamed('localizedReason'),
        options: anyNamed('options'),
      ),
    );
    verify(biometricRepository.enableBiometricLogin()).called(1);
  });
}
