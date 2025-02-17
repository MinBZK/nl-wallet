import 'package:fimber/fimber.dart';

import '../../../../data/repository/network/network_repository.dart';
import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../../util/extension/wallet_instruction_result_extension.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../model/result/application_error.dart';
import '../unlock_wallet_with_pin_usecase.dart';

class UnlockWalletWithPinUseCaseImpl extends UnlockWalletWithPinUseCase {
  final WalletRepository _walletRepository;
  final NetworkRepository _networkRepository;

  UnlockWalletWithPinUseCaseImpl(this._walletRepository, this._networkRepository);

  @override
  Future<Result<String?>> invoke(String pin) async {
    try {
      final result = await _walletRepository.unlockWallet(pin);
      return result.asApplicationResult();
    } on CoreNetworkError catch (ex) {
      Fimber.e('Could not reach server to validate pin', ex: ex);
      final hasInternet = await _networkRepository.hasInternet();
      return Result.error(NetworkError(hasInternet: hasInternet, sourceError: ex));
    } catch (ex) {
      Fimber.e('Failed to unlock with pin', ex: ex);
      return Result.error(GenericError(ex.toString(), sourceError: ex));
    }
  }
}
