import 'package:fimber/fimber.dart';
import 'package:flutter/foundation.dart';
import 'package:local_auth/local_auth.dart';

import '../get_available_biometrics_usecase.dart';

class GetAvailableBiometricsUseCaseImpl extends GetAvailableBiometricsUseCase {
  final LocalAuthentication _localAuthentication;
  final TargetPlatform _platform;

  GetAvailableBiometricsUseCaseImpl(this._localAuthentication, this._platform);

  @override
  Future<AvailableBiometrics> invoke() async {
    final List<BiometricType> availableBiometrics = await _localAuthentication.getAvailableBiometrics();
    Fimber.d('Supported biometrics: $availableBiometrics');
    // Require strong type biometrics (android only)
    if (_platform == TargetPlatform.android && !availableBiometrics.supportsStrongType) return AvailableBiometrics.none;
    if (availableBiometrics.supportsFingerprintAndFaceType) return AvailableBiometrics.some;
    if (availableBiometrics.supportsFingerprintType) return AvailableBiometrics.fingerOnly;
    if (availableBiometrics.supportsFaceType) return AvailableBiometrics.faceOnly;
    if (availableBiometrics.supportsStrongType) return AvailableBiometrics.some;
    return AvailableBiometrics.none;
  }
}

extension _BiometricExtentions on List<BiometricType> {
  bool get supportsStrongType => contains(BiometricType.strong);

  bool get supportsFingerprintAndFaceType => supportsFaceType && supportsFingerprintType;

  bool get supportsFingerprintType => contains(BiometricType.fingerprint);

  bool get supportsFaceType => contains(BiometricType.face);
}
