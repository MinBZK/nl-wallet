import 'package:flutter/foundation.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:local_auth/local_auth.dart';
import 'package:local_auth_android/local_auth_android.dart';
import 'package:local_auth_darwin/local_auth_darwin.dart';

import '../../domain/usecase/biometrics/biometrics.dart';
import '../extension/biometric_type_extension.dart';

class LocalAuthenticationHelper {
  static Future<bool> authenticate(
    LocalAuthentication localAuthentication,
    TargetPlatform targetPlatform,
    AppLocalizations l10n, {
    String? localizedReason,
    bool useErrorDialogs = true,
  }) async {
    return localAuthentication.authenticate(
      authMessages: await _authMessages(localAuthentication, targetPlatform, l10n),
      localizedReason: localizedReason ?? l10n.setupBiometricsPageLocalizedReason,
      options: AuthenticationOptions(
        biometricOnly: true,
        useErrorDialogs: useErrorDialogs,
      ),
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
        biometricHint: switch (biometrics) {
          Biometrics.face => l10n.localAuthMessageAndroidBiometricHintFaceScanOnly,
          Biometrics.fingerprint => l10n.localAuthMessageAndroidBiometricHintFingerPrintOnly,
          _ => l10n.localAuthMessageAndroidBiometricHint,
        },
        biometricNotRecognized: l10n.localAuthMessageAndroidBiometricNotRecognized,
        biometricRequiredTitle: l10n.localAuthMessageAndroidBiometricRequiredTitle,
        biometricSuccess: l10n.localAuthMessageAndroidBiometricSuccess,
        cancelButton: l10n.localAuthMessageAndroidCancelButton,
        deviceCredentialsRequiredTitle: l10n.localAuthMessageAndroidDeviceCredentialsRequiredTitle,
        deviceCredentialsSetupDescription: l10n.localAuthMessageAndroidDeviceCredentialsSetupDescription,
        goToSettingsButton: l10n.localAuthMessageGoToSettingsButton,
        goToSettingsDescription: l10n.localAuthMessageAndroidGoToSettingsDescription,
        signInTitle: switch (biometrics) {
          Biometrics.face => l10n.localAuthMessageAndroidSignInTitleFaceScanOnly,
          Biometrics.fingerprint => l10n.localAuthMessageAndroidSignInTitleFingerPrintOnly,
          _ => l10n.localAuthMessageAndroidSignInTitle,
        },
      ),
      IOSAuthMessages(
        lockOut: l10n.localAuthMessageiOSLockOut,
        goToSettingsButton: l10n.localAuthMessageGoToSettingsButton,
        goToSettingsDescription: l10n.localAuthMessageiOSGoToSettingsDescription,
        cancelButton: l10n.localAuthMessageiOSCancelButton,
      ),
    ];
  }
}
