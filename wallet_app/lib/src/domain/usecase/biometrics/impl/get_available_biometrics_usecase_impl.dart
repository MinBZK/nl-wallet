import 'package:fimber/fimber.dart';
import 'package:flutter/foundation.dart';
import 'package:local_auth/local_auth.dart';

import '../../../../util/extension/biometric_type_extension.dart';
import '../get_available_biometrics_usecase.dart';

/// Docs: [GetAvailableBiometricsUseCase]
class GetAvailableBiometricsUseCaseImpl extends GetAvailableBiometricsUseCase {
  final LocalAuthentication _localAuthentication;
  final TargetPlatform _platform;

  GetAvailableBiometricsUseCaseImpl(this._localAuthentication, this._platform);

  @override
  Future<Biometrics> invoke() async {
    try {
      final List<BiometricType> availableBiometricTypes = await _localAuthentication.getAvailableBiometrics();
      Fimber.d('Supported biometrics: $availableBiometricTypes');
      return availableBiometricTypes.toBiometrics(targetPlatform: _platform);
    } catch (ex) {
      Fimber.e('Could not resolve available biometric types, falling back to Biometrics.none', ex: ex);
      return Biometrics.none;
    }
  }
}
