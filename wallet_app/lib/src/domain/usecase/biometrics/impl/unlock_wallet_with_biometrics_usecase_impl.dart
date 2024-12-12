import 'package:fimber/fimber.dart';
import 'package:flutter/services.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:local_auth/error_codes.dart' as auth_error;
import 'package:local_auth/local_auth.dart';

import '../../../../data/repository/biometric/biometric_repository.dart';
import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../../data/store/active_locale_provider.dart';
import '../../../../util/helper/local_authentication_helper.dart';
import '../biometric_authentication_result.dart';
import '../unlock_wallet_with_biometrics_usecase.dart';

class UnlockWalletWithBiometricsUseCaseImpl extends UnlockWalletWithBiometricsUseCase {
  final BiometricRepository _biometricRepository;
  final ActiveLocaleProvider _localeProvider;
  final LocalAuthentication _localAuthentication;
  final TargetPlatform _targetPlatform;
  final WalletRepository _walletRepository;

  UnlockWalletWithBiometricsUseCaseImpl(
    this._biometricRepository,
    this._localeProvider,
    this._localAuthentication,
    this._targetPlatform,
    this._walletRepository,
  );

  @override
  Future<BiometricAuthenticationResult> invoke() async {
    // Perform sanity checks
    final isEnabled = await _biometricRepository.isBiometricLoginEnabled();
    if (!isEnabled) throw UnsupportedError('Biometric unlock is not enabled');
    final canCheckBiometrics = await _localAuthentication.canCheckBiometrics;
    if (!canCheckBiometrics) throw UnsupportedError('Biometric unlock is not available on this device');
    final availableBiometrics = await _localAuthentication.getAvailableBiometrics();
    if (availableBiometrics.isEmpty) throw UnsupportedError('No available biometrics');

    try {
      final l10n = lookupAppLocalizations(_localeProvider.activeLocale);
      final authenticated = await LocalAuthenticationHelper.authenticate(
        _localAuthentication,
        _targetPlatform,
        l10n,
        localizedReason: l10n.unlockWithBiometricsReason,
        useErrorDialogs: false, /* we never want to allow setup at this stage */
      );
      if (authenticated) {
        await _walletRepository.unlockWalletWithBiometrics();
        return BiometricAuthenticationResult.success;
      }
    } on PlatformException catch (e) {
      if (e.code == auth_error.lockedOut || e.code == auth_error.permanentlyLockedOut) {
        return BiometricAuthenticationResult.lockedOut;
      }
    } catch (ex) {
      Fimber.e('Failed to authenticate', ex: ex);
      return BiometricAuthenticationResult.failure;
    }
    return BiometricAuthenticationResult.failure;
  }
}
