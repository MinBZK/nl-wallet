import 'dart:ui';

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:local_auth/local_auth.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/usecase/biometrics/biometric_authentication_result.dart';
import 'package:wallet/src/domain/usecase/biometrics/impl/request_biometrics_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockLocalAuthentication localAuthentication;
  late MockActiveLocaleProvider localeProvider;

  setUp(() {
    localAuthentication = MockLocalAuthentication();
    localeProvider = MockActiveLocaleProvider();
    when(localeProvider.activeLocale).thenReturn(const Locale('en'));
    when(localAuthentication.getAvailableBiometrics()).thenAnswer((_) async => []);
  });

  void mockAuthenticate({bool authenticated = true, LocalAuthExceptionCode? throws}) {
    when(
      localAuthentication.authenticate(
        authMessages: anyNamed('authMessages'),
        localizedReason: anyNamed('localizedReason'),
        biometricOnly: anyNamed('biometricOnly'),
      ),
    ).thenAnswer((_) async {
      if (throws != null) throw LocalAuthException(code: throws);
      return authenticated;
    });
  }

  Future<Result<BiometricAuthenticationResult>> invoke(TargetPlatform platform) =>
      RequestBiometricsUsecaseImpl(localAuthentication, localeProvider, platform).invoke();

  test('successful authentication results in success', () async {
    mockAuthenticate();
    final result = await invoke(TargetPlatform.iOS);
    expect(result.value, BiometricAuthenticationResult.success);
  });

  test('a denied Face ID permission on iOS results in setupRequired', () async {
    // iOS reports a denied permission as missing hardware, while the device does support biometrics.
    mockAuthenticate(throws: LocalAuthExceptionCode.noBiometricHardware);
    when(localAuthentication.canCheckBiometrics).thenAnswer((_) async => true);

    final result = await invoke(TargetPlatform.iOS);

    expect(result.value, BiometricAuthenticationResult.setupRequired);
  });

  test('unusable biometric hardware on Android results in setupRequired', () async {
    mockAuthenticate(throws: LocalAuthExceptionCode.noBiometricHardware);
    when(localAuthentication.canCheckBiometrics).thenAnswer((_) async => true);

    final result = await invoke(TargetPlatform.android);

    expect(result.value, BiometricAuthenticationResult.setupRequired);
  });

  test('missing biometric hardware results in an error', () async {
    mockAuthenticate(throws: LocalAuthExceptionCode.noBiometricHardware);
    when(localAuthentication.canCheckBiometrics).thenAnswer((_) async => false);

    final result = await invoke(TargetPlatform.iOS);

    expect(result.error, isA<GenericError>());
  });

  test('no enrolled biometrics results in setupRequired', () async {
    mockAuthenticate(throws: LocalAuthExceptionCode.noBiometricsEnrolled);
    when(localAuthentication.canCheckBiometrics).thenAnswer((_) async => true);

    final result = await invoke(TargetPlatform.iOS);

    expect(result.value, BiometricAuthenticationResult.setupRequired);
  });

  test('a biometric lockout results in lockedOut', () async {
    mockAuthenticate(throws: LocalAuthExceptionCode.biometricLockout);
    when(localAuthentication.canCheckBiometrics).thenAnswer((_) async => true);

    final result = await invoke(TargetPlatform.iOS);

    expect(result.value, BiometricAuthenticationResult.lockedOut);
  });

  test('a failed authentication results in an error', () async {
    mockAuthenticate(authenticated: false);

    final result = await invoke(TargetPlatform.iOS);

    expect(result.error, isA<GenericError>());
  });
}
