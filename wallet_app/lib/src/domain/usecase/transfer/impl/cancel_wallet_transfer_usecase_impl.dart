import 'package:fimber/fimber.dart';

import '../../../../data/repository/network/network_repository.dart';
import '../../../../data/repository/transfer/transfer_repository.dart';
import '../../../../util/extension/wallet_instruction_result_extension.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../model/result/application_error.dart';
import '../../../model/result/result.dart';
import '../cancel_wallet_transfer_usecase.dart';

class CancelWalletTransferUseCaseImpl extends CancelWalletTransferUseCase {
  final TransferRepository _transferRepository;
  final NetworkRepository _networkRepository;

  CancelWalletTransferUseCaseImpl(this._transferRepository, this._networkRepository);

  @override
  Future<Result<void>> invoke() async {
    try {
      // TODO(Rob): Check why this returns a WalletIntructionResult (making this code more complex, otherwise tryCatch(...) should suffice.
      final result = await _transferRepository.cancelWalletTransfer();
      return result.asApplicationResult();
    } on CoreNetworkError catch (ex) {
      Fimber.e('Could not reach server', ex: ex);
      final hasInternet = await _networkRepository.hasInternet();
      return Result.error(NetworkError(hasInternet: hasInternet, sourceError: ex));
    } catch (ex) {
      Fimber.e('Failed to cancel wallet transfer', ex: ex);
      return Result.error(GenericError(ex.toString(), sourceError: ex));
    }
  }
}
