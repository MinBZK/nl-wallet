abstract class BiometricRepository {
  Future<bool> isBiometricLoginEnabled();

  Future<void> enableBiometricLogin();

  Future<void> disableBiometricLogin();
}
