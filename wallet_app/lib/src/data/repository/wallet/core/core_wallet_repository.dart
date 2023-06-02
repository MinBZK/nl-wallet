import 'package:core_domain/core_domain.dart';
import 'package:rxdart/rxdart.dart';

import '../../../../domain/model/pin/pin_validation_error.dart';
import '../../../../wallet_core/typed_wallet_core.dart';
import '../../../mapper/mapper.dart';
import '../wallet_repository.dart';

class CoreWalletRepository implements WalletRepository {
  final TypedWalletCore _walletCore;
  final Mapper<PinValidationResult, PinValidationError?> _pinValidationErrorMapper;

  //TODO: This should be moved into the rust core
  final BehaviorSubject<bool> _locked = BehaviorSubject<bool>.seeded(true);

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
    _locked.value = false; // Unlock on creation
  }

  @override
  // TODO: implement confirmTransaction
  Future<bool> confirmTransaction(String pin) => throw UnimplementedError();

  @override
  Future<bool> isRegistered() => _walletCore.isRegistered();

  @override
  Stream<bool> get isLockedStream => _locked;

  @override
  // TODO: implement leftoverPinAttempts
  int get leftoverPinAttempts => throw UnimplementedError();

  @override
  void lockWallet() => _locked.add(true);

  @override
  Future<void> unlockWallet(String pin) async {
    if (_locked.value == false) return; // Already unlocked
    if (await isRegistered() == false) throw UnsupportedError('Wallet not yet registered!');

    ///TODO: Actually check the pin
    _locked.add(false);
  }

  @override
  // TODO: implement destroyWallet
  Future<void> destroyWallet() => throw UnimplementedError();
}
