import 'dart:io';

import 'package:fimber/fimber.dart';

import '../../../../../bridge_generated.dart';
import '../../../../domain/model/pin/pin_validation_error.dart';
import '../../../../domain/usecase/pin/confirm_transaction_usecase.dart';
import '../../../../wallet_core/typed_wallet_core.dart';
import '../../../mapper/mapper.dart';
import '../wallet_repository.dart';

class CoreWalletRepository implements WalletRepository {
  final TypedWalletCore _walletCore;
  final Mapper<PinValidationResult, PinValidationError?> _pinValidationErrorMapper;

  CoreWalletRepository(this._walletCore, this._pinValidationErrorMapper);

  @override
  Future<void> validatePin(String pin) async {
    final result = await _walletCore.isValidPin(pin);
    final error = _pinValidationErrorMapper.map(result);

    if (error != null) {
      throw error;
    }
  }

  @override
  Future<void> createWallet(String pin) async {
    await _walletCore.register(pin);
  }

  @override
  // TODO: implement confirmTransaction
  Future<CheckPinResult> confirmTransaction(String pin) => throw UnimplementedError();

  @override
  Future<bool> isRegistered() => _walletCore.isRegistered();

  @override
  Stream<bool> get isLockedStream => _walletCore.isLocked;

  @override
  void lockWallet() async {
    try {
      await _walletCore.lockWallet();
    } catch (exception) {
      Fimber.e('Failed to lock wallet', ex: exception);
      exit(1); // Crash if locking fails
    }
  }

  @override
  Future<WalletUnlockResult> unlockWallet(String pin) async {
    if (await isRegistered() == false) throw UnsupportedError('Wallet not yet registered!');
    return await _walletCore.unlockWallet(pin);
  }

  @override
  // TODO: implement destroyWallet
  Future<void> destroyWallet() => throw UnimplementedError();
}
