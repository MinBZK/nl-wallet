import 'package:fimber/fimber.dart';

import '../../../../data/repository/network/network_repository.dart';
import '../../../../data/repository/transfer/transfer_repository.dart';
import '../../../../util/extension/wallet_instruction_result_extension.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../model/result/application_error.dart';
import '../../../model/result/result.dart';
import '../prepare_wallet_transfer_usecase.dart';

class PrepareWalletTransferUseCaseImpl extends PrepareWalletTransferUseCase {
  final TransferRepository _transferRepository;
  final NetworkRepository _networkRepository;

  PrepareWalletTransferUseCaseImpl(this._transferRepository, this._networkRepository);

  @override
  Future<Result<void>> invoke(String pin) async {
    try {
      final result = await _transferRepository.prepareTransferWallet(pin);
      return result.asApplicationResult();
    } on CoreNetworkError catch (ex) {
      Fimber.e('Could not reach server to validate pin', ex: ex);
      final hasInternet = await _networkRepository.hasInternet();
      return Result.error(NetworkError(hasInternet: hasInternet, sourceError: ex));
    } catch (ex) {
      Fimber.e('Failed to transfer wallet', ex: ex);
      return Result.error(GenericError(ex.toString(), sourceError: ex));
    }
  }
}
