import 'package:fimber/fimber.dart';
import 'package:flutter/services.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:local_auth/error_codes.dart' as auth_error;
import 'package:local_auth/local_auth.dart';

import '../../../../data/store/active_locale_provider.dart';
import '../request_biometrics_usecase.dart';

class RequestBiometricsUsecaseImpl extends RequestBiometricsUsecase {
  final LocalAuthentication _localAuthentication;
  final ActiveLocaleProvider _localeProvider;
  final TargetPlatform _targetPlatform;

  RequestBiometricsUsecaseImpl(this._localAuthentication, this._localeProvider, this._targetPlatform);

  @override
  Future<RequestBiometricsResult> invoke() async {
    final l10n = lookupAppLocalizations(_localeProvider.activeLocale);
    try {
      final authenticated = await _localAuthentication.authenticate(
        localizedReason: l10n.setupBiometricsPageLocalizedReason,
        options: const AuthenticationOptions(biometricOnly: true, useErrorDialogs: true),
      );
      if (authenticated) return RequestBiometricsResult.success;
    } on PlatformException catch (e) {
      final canCheckBiometrics = await _localAuthentication.canCheckBiometrics;
      if (e.code == auth_error.notAvailable) {
        Fimber.e('Not available. Supports biometrics: $canCheckBiometrics', ex: e);
        // "if" check to cover issue where android reports not available due to incorrect system setting (https://github.com/flutter/flutter/issues/96646)
        if (canCheckBiometrics && _targetPlatform == TargetPlatform.android) {
          return RequestBiometricsResult.setupRequired;
        }
      } else if (e.code == auth_error.notEnrolled) {
        Fimber.e('Not enrolled. Supports biometrics: $canCheckBiometrics', ex: e);
        return RequestBiometricsResult.setupRequired;
      } else {
        Fimber.e('Other PlatformException', ex: e);
      }
    }
    return RequestBiometricsResult.failure;
  }
}
