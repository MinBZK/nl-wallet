import 'package:local_auth/local_auth.dart';

extension BiometricTypeExtention on List<BiometricType> {
  bool get supportsStrongType => contains(BiometricType.strong);

  bool get supportsFingerprintAndFaceType => supportsFaceType && supportsFingerprintType;

  bool get supportsFingerprintType => contains(BiometricType.fingerprint);

  bool get supportsFaceType => contains(BiometricType.face);
}
