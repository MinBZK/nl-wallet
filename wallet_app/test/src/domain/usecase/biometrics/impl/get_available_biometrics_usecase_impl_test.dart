import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:local_auth/local_auth.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_available_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/impl/get_available_biometrics_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  test('[android] returns face when strong & face type biometrics is available', () async {
    final mockLocalAuthentication = MockLocalAuthentication();
    when(mockLocalAuthentication.getAvailableBiometrics())
        .thenAnswer((_) async => [BiometricType.strong, BiometricType.face]);
    final usecase = GetAvailableBiometricsUseCaseImpl(
      mockLocalAuthentication,
      TargetPlatform.android,
    );
    final result = await usecase.invoke();
    expect(result, AvailableBiometrics.faceOnly);
  });

  test('[android] returns finger when strong & finger type biometrics is available', () async {
    final mockLocalAuthentication = MockLocalAuthentication();
    when(mockLocalAuthentication.getAvailableBiometrics())
        .thenAnswer((_) async => [BiometricType.strong, BiometricType.fingerprint]);
    final usecase = GetAvailableBiometricsUseCaseImpl(
      mockLocalAuthentication,
      TargetPlatform.android,
    );
    final result = await usecase.invoke();
    expect(result, AvailableBiometrics.fingerOnly);
  });

  test('[android] returns some when all biometric types are available', () async {
    final mockLocalAuthentication = MockLocalAuthentication();
    when(mockLocalAuthentication.getAvailableBiometrics()).thenAnswer((_) async => BiometricType.values);
    final usecase = GetAvailableBiometricsUseCaseImpl(
      mockLocalAuthentication,
      TargetPlatform.android,
    );
    final result = await usecase.invoke();
    expect(result, AvailableBiometrics.some);
  });

  test('[android] returns none when strong type biometrics are not available', () async {
    final mockLocalAuthentication = MockLocalAuthentication();
    when(mockLocalAuthentication.getAvailableBiometrics()).thenAnswer(
      (_) async => [
        BiometricType.face,
        BiometricType.fingerprint,
        BiometricType.iris,
        BiometricType.weak,
      ],
    );
    final usecase = GetAvailableBiometricsUseCaseImpl(
      mockLocalAuthentication,
      TargetPlatform.android,
    );

    final result = await usecase.invoke();
    expect(result, AvailableBiometrics.none);
  });

  test('[android] returns none when no biometrics are available', () async {
    final mockLocalAuthentication = MockLocalAuthentication();
    when(mockLocalAuthentication.getAvailableBiometrics()).thenAnswer((_) async => []);
    final usecase = GetAvailableBiometricsUseCaseImpl(
      mockLocalAuthentication,
      TargetPlatform.android,
    );

    final result = await usecase.invoke();
    expect(result, AvailableBiometrics.none);
  });

  test('[android] returns some when only strong biometrics are available', () async {
    final mockLocalAuthentication = MockLocalAuthentication();
    when(mockLocalAuthentication.getAvailableBiometrics()).thenAnswer((_) async => [BiometricType.strong]);
    final usecase = GetAvailableBiometricsUseCaseImpl(
      mockLocalAuthentication,
      TargetPlatform.android,
    );

    final result = await usecase.invoke();
    expect(result, AvailableBiometrics.some);
  });

  test('[iOS] returns face when face type biometrics is available', () async {
    final mockLocalAuthentication = MockLocalAuthentication();
    when(mockLocalAuthentication.getAvailableBiometrics()).thenAnswer((_) async => [BiometricType.face]);
    final usecase = GetAvailableBiometricsUseCaseImpl(
      mockLocalAuthentication,
      TargetPlatform.iOS,
    );
    final result = await usecase.invoke();
    expect(result, AvailableBiometrics.faceOnly);
  });

  test('[iOS] returns finger when fingerprint type biometrics is available', () async {
    final mockLocalAuthentication = MockLocalAuthentication();
    when(mockLocalAuthentication.getAvailableBiometrics()).thenAnswer((_) async => [BiometricType.fingerprint]);
    final usecase = GetAvailableBiometricsUseCaseImpl(
      mockLocalAuthentication,
      TargetPlatform.iOS,
    );
    final result = await usecase.invoke();
    expect(result, AvailableBiometrics.fingerOnly);
  });

  test('[iOS] returns some when all biometric types are available', () async {
    final mockLocalAuthentication = MockLocalAuthentication();
    when(mockLocalAuthentication.getAvailableBiometrics()).thenAnswer((_) async => BiometricType.values);
    final usecase = GetAvailableBiometricsUseCaseImpl(
      mockLocalAuthentication,
      TargetPlatform.iOS,
    );
    final result = await usecase.invoke();
    expect(result, AvailableBiometrics.some);
  });

  test('[iOS] returns some when only face and finger biometric types are available', () async {
    final mockLocalAuthentication = MockLocalAuthentication();
    when(mockLocalAuthentication.getAvailableBiometrics())
        .thenAnswer((_) async => [BiometricType.face, BiometricType.fingerprint]);
    final usecase = GetAvailableBiometricsUseCaseImpl(
      mockLocalAuthentication,
      TargetPlatform.iOS,
    );
    final result = await usecase.invoke();
    expect(result, AvailableBiometrics.some);
  });

  test('[iOS] returns none when no biometrics are available', () async {
    final mockLocalAuthentication = MockLocalAuthentication();
    when(mockLocalAuthentication.getAvailableBiometrics()).thenAnswer((_) async => []);
    final usecase = GetAvailableBiometricsUseCaseImpl(
      mockLocalAuthentication,
      TargetPlatform.iOS,
    );

    final result = await usecase.invoke();
    expect(result, AvailableBiometrics.none);
  });
}
