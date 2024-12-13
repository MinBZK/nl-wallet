import 'package:flutter/foundation.dart';
import 'package:local_auth/local_auth.dart';

import '../../domain/usecase/biometrics/biometrics.dart';

extension BiometricTypeExtention on List<BiometricType> {
  bool get supportsStrongType => contains(BiometricType.strong);

  bool get supportsFingerprintAndFaceType => supportsFaceType && supportsFingerprintType;

  bool get supportsFingerprintType => contains(BiometricType.fingerprint);

  bool get supportsFaceType => contains(BiometricType.face);

  Biometrics toBiometrics({required TargetPlatform targetPlatform}) {
    // Require strong type biometrics (android only)
    if (targetPlatform == TargetPlatform.android && !supportsStrongType) return Biometrics.none;
    if (supportsFingerprintAndFaceType) return Biometrics.some;
    if (supportsFingerprintType) return Biometrics.fingerprint;
    if (supportsFaceType) return Biometrics.face;
    if (supportsStrongType) return Biometrics.some;
    return Biometrics.none;
  }
}
