import 'package:core_domain/core_domain.dart';
import 'package:rxdart/rxdart.dart';

import '../../../../domain/model/pin/pin_validation_error.dart';
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
    _walletCore.register(pin);
  }

  @override
  Future<bool> confirmTransaction(String pin) {
    // TODO: implement confirmTransaction
    throw UnimplementedError();
  }

  @override
  // TODO: implement isInitializedStream (default to false until createWallet is implemented)
  Stream<bool> get isInitializedStream => BehaviorSubject.seeded(false);

  @override
  // TODO: implement isLockedStream
  Stream<bool> get isLockedStream => throw UnimplementedError();

  @override
  // TODO: implement leftoverPinAttempts
  int get leftoverPinAttempts => throw UnimplementedError();

  @override
  void lockWallet() {
    // TODO: implement lockWallet
    throw UnimplementedError();
  }

  @override
  void unlockWallet(String pin) {
    // TODO: implement unlockWallet
    throw UnimplementedError();
  }

  @override
  Future<void> destroyWallet() {
    // TODO: implement destroyWallet
    throw UnimplementedError();
  }
}
