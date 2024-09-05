/// Enable/Disable the usage of biometrics for
/// unlocking the app. Signing transactions will
/// still require the user's PIN.
abstract class SetBiometricsUseCase {
  Future<void> invoke({required bool enable, required bool authenticateBeforeEnabling});
}
