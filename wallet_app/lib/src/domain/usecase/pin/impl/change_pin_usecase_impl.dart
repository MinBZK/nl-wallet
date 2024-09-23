import 'dart:async';

import '../../../../data/repository/wallet/wallet_repository.dart';
import '../change_pin_usecase.dart';

class ChangePinUseCaseImpl extends ChangePinUseCase {
  final WalletRepository walletRepository;

  ChangePinUseCaseImpl(this.walletRepository);

  @override
  Future<void> invoke(String oldPin, String newPin) async {
    try {
      final result = await walletRepository.changePin(oldPin, newPin);
      result.when(
        ok: () => unawaited(walletRepository.continueChangePin(newPin)),
        instructionError: (error) {
          throw StateError(
            'WalletInstructionResult should not occur here, as validation is checked in the flow. $error',
          );
        },
      );
    } catch (ex) {
      unawaited(walletRepository.continueChangePin(oldPin));
      rethrow;
    }
  }
}
