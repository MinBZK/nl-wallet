import 'package:fimber/fimber.dart';
import 'package:local_auth/local_auth.dart';

import '../../../../util/extension/biometric_type_extension.dart';
import '../get_supported_biometrics_usecase.dart';

/// Docs: [GetSupportedBiometricsUseCase]
class GetSupportedBiometricsUseCaseImpl extends GetSupportedBiometricsUseCase {
  final LocalAuthentication _localAuthentication;

  GetSupportedBiometricsUseCaseImpl(this._localAuthentication);

  @override
  Future<Biometrics> invoke() async {
    try {
      final canCheckBiometrics = await _localAuthentication.canCheckBiometrics;
      if (!canCheckBiometrics) return Biometrics.none;

      // Try to resolve more specific info, though this list can be empty when biometrics are not yet configured.
      final List<BiometricType> availableBiometrics = await _localAuthentication.getAvailableBiometrics();
      if (availableBiometrics.supportsFingerprintAndFaceType) return Biometrics.some;
      if (availableBiometrics.supportsFaceType) return Biometrics.face;
      if (availableBiometrics.supportsFingerprintType) return Biometrics.fingerprint;

      // Default to generic 'some' when [canCheckBiometrics] is true but further details can't be resolved.
      return Biometrics.some;
    } catch (ex) {
      Fimber.e('Could not resolve supported biometric types, falling back to Biometrics.none', ex: ex);
      return Biometrics.none;
    }
  }
}
