import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:wallet_core/core.dart';

import '../../../../data/repository/wallet/wallet_repository.dart';
import '../change_pin_usecase.dart';

class ChangePinUseCaseImpl extends ChangePinUseCase {
  final WalletRepository walletRepository;

  ChangePinUseCaseImpl(this.walletRepository);

  @override
  Future<void> invoke(String oldPin, String newPin) async {
    bool pinUpdated = false;
    try {
      final result = await walletRepository.changePin(oldPin, newPin);
      pinUpdated = result is WalletInstructionResult_Ok;
    } catch (ex) {
      rethrow;
    } finally {
      continueChangePin(pinUpdated ? newPin : oldPin);
    }
  }

  void continueChangePin(String pin) {
    unawaited(
      walletRepository.continueChangePin(pin).then(
        (v) {
          Fimber.d('Successfully notified server about commit/rollback');
        },
        onError: (ex) {
          Fimber.e('Failed to commit/rollback', ex: ex);
        },
      ),
    );
  }
}
