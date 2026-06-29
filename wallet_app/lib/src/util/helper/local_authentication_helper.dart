import 'package:flutter/foundation.dart';
import 'package:local_auth/local_auth.dart';
import 'package:local_auth_android/local_auth_android.dart';
import 'package:local_auth_darwin/local_auth_darwin.dart';

import '../../../l10n/generated/app_localizations.dart';
import '../../domain/usecase/biometrics/biometrics.dart';
import '../extension/biometric_type_extension.dart';

class LocalAuthenticationHelper {
  static Future<bool> authenticate(
    LocalAuthentication localAuthentication,
    TargetPlatform targetPlatform,
    AppLocalizations l10n, {
    String? localizedReason,
  }) async {
    return localAuthentication.authenticate(
      authMessages: await _authMessages(localAuthentication, targetPlatform, l10n),
      localizedReason: localizedReason ?? l10n.setupBiometricsPageLocalizedReason,
      biometricOnly: true,
    );
  }

  static Future<List<AuthMessages>> _authMessages(
    LocalAuthentication localAuthentication,
    TargetPlatform targetPlatform,
    AppLocalizations l10n,
  ) async {
    final availableBiometrics = await localAuthentication.getAvailableBiometrics();
    final biometrics = availableBiometrics.toBiometrics(targetPlatform: targetPlatform);
    return [
      AndroidAuthMessages(
        signInHint: switch (biometrics) {
          Biometrics.face => l10n.localAuthMessageAndroidBiometricHintFaceScanOnly,
          Biometrics.fingerprint => l10n.localAuthMessageAndroidBiometricHintFingerPrintOnly,
          _ => l10n.localAuthMessageAndroidBiometricHint,
        },
        cancelButton: l10n.localAuthMessageAndroidCancelButton,
        signInTitle: switch (biometrics) {
          Biometrics.face => l10n.localAuthMessageAndroidSignInTitleFaceScanOnly,
          Biometrics.fingerprint => l10n.localAuthMessageAndroidSignInTitleFingerPrintOnly,
          _ => l10n.localAuthMessageAndroidSignInTitle,
        },
      ),
      IOSAuthMessages(
        cancelButton: l10n.localAuthMessageiOSCancelButton,
      ),
    ];
  }
}
