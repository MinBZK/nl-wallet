import 'get_available_biometrics_usecase.dart';

export 'biometrics.dart';

/// Get the biometrics that are supported by the device, this does not mean they are configured.
/// For configured biometrics rely on [GetAvailableBiometricsUseCase].
abstract class GetSupportedBiometricsUseCase {
  Future<Biometrics> invoke();
}
