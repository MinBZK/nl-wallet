import 'biometrics.dart';

export 'biometrics.dart';

/// Check if the device supports (secure) biometrics and if they are configured by the user.
/// If so, it reports which variant (face/finger) is setup, falling back to 'some' if these
/// details are unavailable.
/// If you only want to check if the device supports biometrics and don't care about the
/// availability (i.e. if they are set up by the user), see [GetSupportedBiometricsUseCase].
abstract class GetAvailableBiometricsUseCase {
  Future<Biometrics> invoke();
}
