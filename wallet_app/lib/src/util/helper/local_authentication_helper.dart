import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:local_auth/local_auth.dart';
import 'package:local_auth_android/local_auth_android.dart';
import 'package:local_auth_darwin/local_auth_darwin.dart';

class LocalAuthenticationHelper {
  static Future<bool> authenticate(
    LocalAuthentication localAuthentication,
    AppLocalizations l10n, {
    String? localizedReason,
    bool useErrorDialogs = true,
  }) {
    return localAuthentication.authenticate(
      authMessages: _authMessages(l10n),
      localizedReason: localizedReason ?? l10n.setupBiometricsPageLocalizedReason,
      options: AuthenticationOptions(
        biometricOnly: true,
        useErrorDialogs: useErrorDialogs,
      ),
    );
  }

  static List<AuthMessages> _authMessages(AppLocalizations l10n) {
    return [
      AndroidAuthMessages(
        biometricHint: l10n.localAuthMessageAndroidBiometricHint,
        biometricNotRecognized: l10n.localAuthMessageAndroidBiometricNotRecognized,
        biometricRequiredTitle: l10n.localAuthMessageAndroidBiometricRequiredTitle,
        biometricSuccess: l10n.localAuthMessageAndroidBiometricSuccess,
        cancelButton: l10n.localAuthMessageAndroidCancelButton,
        deviceCredentialsRequiredTitle: l10n.localAuthMessageAndroidDeviceCredentialsRequiredTitle,
        deviceCredentialsSetupDescription: l10n.localAuthMessageAndroidDeviceCredentialsSetupDescription,
        goToSettingsButton: l10n.localAuthMessageGoToSettingsButton,
        goToSettingsDescription: l10n.localAuthMessageAndroidGoToSettingsDescription,
        signInTitle: l10n.localAuthMessageAndroidSignInTitle,
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
