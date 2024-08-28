/// Check if the device supports (secure) biometrics and if they are configured by the user.
/// If so, it reports which variant (face/finger) is setup, falling back to 'some' if these
/// details are unavailable.
abstract class GetAvailableBiometricsUseCase {
  Future<AvailableBiometrics> invoke();
}

enum AvailableBiometrics { faceOnly, fingerOnly, some, none }
