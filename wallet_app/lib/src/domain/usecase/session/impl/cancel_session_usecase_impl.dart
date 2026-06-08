import 'package:fimber/fimber.dart';

import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../../util/extension/core_error_extension.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../model/result/application_error.dart';
import '../../../model/result/result.dart';
import '../../../model/wallet_state.dart';
import '../cancel_session_usecase.dart';

class CancelSessionUseCaseImpl extends CancelSessionUseCase {
  final WalletRepository _walletRepository;

  CancelSessionUseCaseImpl(this._walletRepository);

  @override
  Future<Result<String?>> invoke() async {
    try {
      final state = await _walletRepository.getWalletState();
      switch (state) {
        case WalletStateInDisclosureFlow():
        case WalletStateInIssuanceFlow():
          return Result.success(await _walletRepository.cancelSession());
        default:
          Fimber.d('Nothing to cancel from $state');
          return const Result.success(null);
      }
    } on CoreError catch (ex) {
      final returnUrl = ex.returnUrl;
      // This matches the behaviour we previously had in disclosure_bloc, i.e. if the error still gives us a
      // returnUrl, we consider the session cancelled and redirect the user.
      if (returnUrl != null) return Result.success(returnUrl);
      return Result.error(await ex.asApplicationError());
    } catch (ex) {
      Fimber.e('Failed to cancel active session', ex: ex);
      return Result.error(GenericError(ex.toString(), sourceError: ex));
    }
  }
}
