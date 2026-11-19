import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/usecase/biometrics/impl/set_biometrics_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/biometrics/set_biometrics_usecase.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockLocalAuthentication localAuthentication;
  late MockBiometricRepository biometricRepository;
  late SetBiometricsUseCase setBiometricsUseCase;

  setUp(() {
    localAuthentication = MockLocalAuthentication();
    biometricRepository = MockBiometricRepository();
    setBiometricsUseCase = SetBiometricsUseCaseImpl(localAuthentication, biometricRepository);
  });

  test('Verify device is checked for compatibility when enabling', () async {
    when(localAuthentication.canCheckBiometrics).thenAnswer((_) async => true);
    await setBiometricsUseCase.invoke(enable: true);
    verify(localAuthentication.canCheckBiometrics).called(1);
  });

  test('Verify setting to true fails when device is not supported', () async {
    when(localAuthentication.canCheckBiometrics).thenAnswer((_) async => false);

    final result = await setBiometricsUseCase.invoke(enable: true);
    expect(
      result,
      isA<Error>().having(
        (Error e) => e.error,
        'Expecting a HardwareUnsupportedError',
        isA<HardwareUnsupportedError>(),
      ),
    );

    verifyNever(biometricRepository.enableBiometricLogin());
  });

  test('Verify setting to false succeeds even when device is not supported', () async {
    when(localAuthentication.canCheckBiometrics).thenAnswer((_) async => false);
    await setBiometricsUseCase.invoke(enable: false);
    verify(biometricRepository.disableBiometricLogin()).called(1);
  });

  test('Verify setting to true enables biometric login without authenticating', () async {
    when(localAuthentication.canCheckBiometrics).thenAnswer((_) async => true);
    await setBiometricsUseCase.invoke(enable: true);
    verifyNever(
      localAuthentication.authenticate(
        authMessages: anyNamed('authMessages'),
        localizedReason: anyNamed('localizedReason'),
        biometricOnly: anyNamed('biometricOnly'),
      ),
    );
    verify(biometricRepository.enableBiometricLogin()).called(1);
  });
}
