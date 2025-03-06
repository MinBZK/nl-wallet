import 'package:fimber/fimber.dart';
import 'package:flutter/services.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:local_auth/error_codes.dart' as auth_error;
import 'package:local_auth/local_auth.dart';

import '../../../../data/repository/biometric/biometric_repository.dart';
import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../../data/store/active_locale_provider.dart';
import '../../../../util/extension/core_error_extension.dart';
import '../../../../util/helper/local_authentication_helper.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../model/result/application_error.dart';
import '../../../model/result/result.dart';
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
  Future<Result<BiometricAuthenticationResult>> invoke() async {
    try {
      // Perform sanity checks
      final isEnabled = await _biometricRepository.isBiometricLoginEnabled();
      if (!isEnabled) throw UnsupportedError('Biometric unlock is not enabled');
      final canCheckBiometrics = await _localAuthentication.canCheckBiometrics;
      if (!canCheckBiometrics) {
        return Result.error(HardwareUnsupportedError(sourceError: Exception('Can not check biometrics')));
      }
      final availableBiometrics = await _localAuthentication.getAvailableBiometrics();
      if (availableBiometrics.isEmpty) {
        return Result.error(HardwareUnsupportedError(sourceError: Exception('No available biometrics')));
      }

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
        return const Result.success(BiometricAuthenticationResult.success);
      }
    } on PlatformException catch (e) {
      if (e.code == auth_error.lockedOut || e.code == auth_error.permanentlyLockedOut) {
        return const Result.success(BiometricAuthenticationResult.lockedOut);
      }
    } on CoreError catch (ex) {
      Fimber.e('Failed to authenticate', ex: ex);
      return Result.error(await ex.asApplicationError());
    } catch (ex) {
      Fimber.e('Failed to authenticate', ex: ex);
      return Result.error(GenericError(ex.toString(), sourceError: ex));
    }
    return Result.error(
      GenericError(
        'Failed to unlock wallet with biometrics',
        sourceError: Exception('Authentication failed'),
      ),
    );
  }
}
