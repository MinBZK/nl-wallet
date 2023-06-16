import 'package:core_domain/core_domain.dart';

import '../../domain/usecase/pin/check_pin_usecase.dart';

extension WalletUnlockResultExtension on WalletUnlockResult {
  void when({
    Function(WalletUnlockResultOk)? onWalletUnlockResultOk,
    Function(WalletUnlockResultIncorrectPin)? onWalletUnlockResultIncorrectPin,
    Function(WalletUnlockResultTimeout)? onWalletUnlockResultTimeout,
    Function(WalletUnlockResultBlocked)? onWalletUnlockResultBlocked,
    Function(WalletUnlockResultServerError)? onWalletUnlockResultServerError,
  }) {
    if (this is WalletUnlockResultOk) {
      onWalletUnlockResultOk?.call(this as WalletUnlockResultOk);
    } else if (this is WalletUnlockResultIncorrectPin) {
      onWalletUnlockResultIncorrectPin?.call(this as WalletUnlockResultIncorrectPin);
    } else if (this is WalletUnlockResultTimeout) {
      onWalletUnlockResultTimeout?.call(this as WalletUnlockResultTimeout);
    } else if (this is WalletUnlockResultServerError) {
      onWalletUnlockResultServerError?.call(this as WalletUnlockResultServerError);
    } else if (this is WalletUnlockResultBlocked) {
      onWalletUnlockResultBlocked?.call(this as WalletUnlockResultBlocked);
    }
  }

  CheckPinResult asCheckPinResult() {
    if (this is WalletUnlockResultOk) {
      return CheckPinResultOk();
    } else if (this is WalletUnlockResultIncorrectPin) {
      final incorrectResult = (this as WalletUnlockResultIncorrectPin);
      return CheckPinResultIncorrect(
        leftoverAttempts: incorrectResult.leftoverAttempts,
        isFinalAttempt: incorrectResult.isFinalAttempt,
      );
    } else if (this is WalletUnlockResultTimeout) {
      return CheckPinResultTimeout(
        timeoutMillis: (this as WalletUnlockResultTimeout).timeoutMillis,
      );
    } else if (this is WalletUnlockResultServerError) {
      return CheckPinResultServerError();
    } else if (this is WalletUnlockResultBlocked) {
      return CheckPinResultBlocked();
    }
    throw UnsupportedError('Unknown wallet result: $this');
  }
}
