import 'package:fimber/fimber.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:local_auth/local_auth.dart';

import '../../../../data/repository/biometric/biometric_repository.dart';
import '../../../../data/store/active_locale_provider.dart';
import '../../../../util/helper/local_authentication_helper.dart';
import '../set_biometrics_usecase.dart';

class SetBiometricsUseCaseImpl extends SetBiometricsUseCase {
  final LocalAuthentication _localAuthentication;
  final ActiveLocaleProvider _localeProvider;
  final BiometricRepository _biometricRepository;

  SetBiometricsUseCaseImpl(this._localAuthentication, this._localeProvider, this._biometricRepository);

  @override
  Future<void> invoke({required bool enable, required bool authenticateBeforeEnabling}) async {
    final canCheckBiometrics = await _localAuthentication.canCheckBiometrics;
    if (enable && !canCheckBiometrics) throw UnsupportedError('Device does not support biometrics');

    if (enable && authenticateBeforeEnabling) {
      final l10n = lookupAppLocalizations(_localeProvider.activeLocale);
      final authenticated = await LocalAuthenticationHelper.authenticate(
        _localAuthentication,
        l10n,
      );
      if (!authenticated) throw Exception('Failed to enable biometrics, failed to authenticate');
    }

    if (enable) {
      await _biometricRepository.enableBiometricLogin();
      Fimber.d('Biometric login enabled');
    } else {
      await _biometricRepository.disableBiometricLogin();
      Fimber.d('Biometric login disabled');
    }
  }
}
