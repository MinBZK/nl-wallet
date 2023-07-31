import 'package:fimber/fimber.dart';

import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../../util/extension/wallet_unlock_result_extension.dart';
import '../../../../wallet_core/error/flutter_api_error.dart';
import '../unlock_wallet_with_pin_usecase.dart';

class UnlockWalletWithPinUseCaseImpl extends UnlockWalletWithPinUseCase {
  final WalletRepository walletRepository;

  UnlockWalletWithPinUseCaseImpl(this.walletRepository);

  @override
  Future<CheckPinResult> invoke(String pin) async {
    try {
      final result = await walletRepository.unlockWallet(pin);
      return result.asCheckPinResult();
    } catch (ex) {
      Fimber.e('Failed to unlock wallet', ex: ex);
      if (ex is FlutterApiError) {
        switch (ex.type) {
          case FlutterApiErrorType.generic:
            return CheckPinResultGenericError();
          case FlutterApiErrorType.networking:
            return CheckPinResultServerError(null /* TODO: add statusCode */);
        }
      }
      rethrow;
    }
  }
}
