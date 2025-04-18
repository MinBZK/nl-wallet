import 'package:fimber/fimber.dart';
import 'package:flutter/services.dart';
import 'package:local_auth/error_codes.dart' as auth_error;
import 'package:local_auth/local_auth.dart';

import '../../../../../l10n/generated/app_localizations.dart';
import '../../../../data/store/active_locale_provider.dart';
import '../../../../util/helper/local_authentication_helper.dart';
import '../../../model/result/application_error.dart';
import '../../../model/result/result.dart';
import '../biometric_authentication_result.dart';
import '../request_biometrics_usecase.dart';

const _kDefaultErrorMessage = 'Failed to authenticate with biometrics';

class RequestBiometricsUsecaseImpl extends RequestBiometricsUseCase {
  final LocalAuthentication _localAuthentication;
  final ActiveLocaleProvider _localeProvider;
  final TargetPlatform _targetPlatform;

  RequestBiometricsUsecaseImpl(this._localAuthentication, this._localeProvider, this._targetPlatform);

  @override
  Future<Result<BiometricAuthenticationResult>> invoke() async {
    final l10n = lookupAppLocalizations(_localeProvider.activeLocale);
    try {
      final authenticated = await LocalAuthenticationHelper.authenticate(
        _localAuthentication,
        _targetPlatform,
        l10n,
      );
      if (authenticated) return const Result.success(BiometricAuthenticationResult.success);
    } on PlatformException catch (e) {
      final canCheckBiometrics = await _localAuthentication.canCheckBiometrics;
      if (e.code == auth_error.notAvailable) {
        Fimber.e('Not available. Supports biometrics: $canCheckBiometrics', ex: e);
        // "if" check to cover issue where android reports not available due to incorrect system setting (https://github.com/flutter/flutter/issues/96646)
        if (canCheckBiometrics && _targetPlatform == TargetPlatform.android) {
          return const Result.success(BiometricAuthenticationResult.setupRequired);
        }
      } else if (e.code == auth_error.notEnrolled) {
        Fimber.e('Not enrolled. Supports biometrics: $canCheckBiometrics', ex: e);
        return const Result.success(BiometricAuthenticationResult.setupRequired);
      } else if (e.code == auth_error.lockedOut || e.code == auth_error.permanentlyLockedOut) {
        return const Result.success(BiometricAuthenticationResult.lockedOut);
      } else {
        Fimber.e('Other PlatformException', ex: e);
        return Result.error(GenericError(_kDefaultErrorMessage, sourceError: e));
      }
    } catch (ex) {
      Fimber.e('Failed to authenticate', ex: ex);
      return Result.error(GenericError(_kDefaultErrorMessage, sourceError: ex));
    }
    return Result.error(GenericError(_kDefaultErrorMessage, sourceError: Exception('Auth failed')));
  }
}
