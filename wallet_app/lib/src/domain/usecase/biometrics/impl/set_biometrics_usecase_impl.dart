import 'package:fimber/fimber.dart';
import 'package:flutter/cupertino.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:local_auth/local_auth.dart';

import '../../../../data/repository/biometric/biometric_repository.dart';
import '../../../../data/store/active_locale_provider.dart';
import '../../../../util/extension/core_error_extension.dart';
import '../../../../util/helper/local_authentication_helper.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../model/result/application_error.dart';
import '../../../model/result/result.dart';
import '../set_biometrics_usecase.dart';

class SetBiometricsUseCaseImpl extends SetBiometricsUseCase {
  final LocalAuthentication _localAuthentication;
  final TargetPlatform _targetPlatform;
  final ActiveLocaleProvider _localeProvider;
  final BiometricRepository _biometricRepository;

  SetBiometricsUseCaseImpl(
    this._localAuthentication,
    this._targetPlatform,
    this._localeProvider,
    this._biometricRepository,
  );

  @override
  Future<Result<void>> invoke({required bool enable, required bool authenticateBeforeEnabling}) async {
    try {
      // Check if device supports biometrics
      final canCheckBiometrics = await _localAuthentication.canCheckBiometrics;
      if (enable && !canCheckBiometrics) {
        return Result.error(HardwareUnsupportedError(sourceError: Exception('Device unsupported')));
      }

      // Authenticate with biometrics if requested
      if (enable && authenticateBeforeEnabling) {
        final l10n = lookupAppLocalizations(_localeProvider.activeLocale);
        final authenticated = await LocalAuthenticationHelper.authenticate(
          _localAuthentication,
          _targetPlatform,
          l10n,
        );
        if (!authenticated) {
          return Result.error(
            GenericError(
              'Biometrics not enabled: failed to authenticate',
              sourceError: Exception('Authentication failed'),
            ),
          );
        }
      }

      // Toggle the biometric login setting
      if (enable) {
        await _biometricRepository.enableBiometricLogin();
        Fimber.d('Biometric login enabled');
      } else {
        await _biometricRepository.disableBiometricLogin();
        Fimber.d('Biometric login disabled');
      }
      return const Result.success(null);
    } on CoreError catch (ex) {
      Fimber.e('Set biometrics config failed', ex: ex);
      return Result.error(await ex.asApplicationError());
    } catch (ex) {
      Fimber.e('Set biometrics config failed', ex: ex);
      return Result.error(GenericError(ex.toString(), sourceError: ex));
    }
  }
}
