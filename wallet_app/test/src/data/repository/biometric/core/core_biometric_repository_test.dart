import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/biometric/core/core_biometric_repository.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockTypedWalletCore mockTypedWalletCore;
  late CoreBiometricRepository repository;

  setUp(() {
    mockTypedWalletCore = MockTypedWalletCore();
    repository = CoreBiometricRepository(mockTypedWalletCore);
  });

  test('calling enableBiometricLogin enabled it on core', () async {
    await repository.enableBiometricLogin();
    verify(mockTypedWalletCore.setBiometricUnlock(enabled: true)).called(1);
  });

  test('calling disableBiometricLogin disables it on core', () async {
    await repository.disableBiometricLogin();
    verify(mockTypedWalletCore.setBiometricUnlock(enabled: false)).called(1);
  });

  test('biometric state is fetched through core', () async {
    when(mockTypedWalletCore.isBiometricLoginEnabled()).thenAnswer((_) async => true);
    final result = await repository.isBiometricLoginEnabled();
    expect(result, isTrue);
  });
}
