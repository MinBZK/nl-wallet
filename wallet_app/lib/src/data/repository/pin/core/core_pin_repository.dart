import 'package:wallet_core/core.dart';

import '../../../../domain/model/pin/pin_validation_error.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../pin_repository.dart';

class CorePinRepository implements PinRepository {
  final TypedWalletCore _walletCore;
  final Mapper<PinValidationResult, PinValidationError?> _pinValidationErrorMapper;

  CorePinRepository(this._walletCore, this._pinValidationErrorMapper);

  @override
  Future<void> validatePin(String pin) async {
    final result = await _walletCore.isValidPin(pin);
    final error = _pinValidationErrorMapper.map(result);
    if (error != null) {
      throw error;
    }
  }

  @override
  Future<WalletInstructionResult> checkPin(String pin) async {
    await _checkRegistration();
    return _walletCore.checkPin(pin);
  }

  @override
  Future<String> createPinRecoveryRedirectUri() => _walletCore.createPinRecoveryRedirectUri();

  @override
  Future<void> continuePinRecovery(String uri) => _walletCore.continuePinRecovery(uri);

  @override
  Future<void> completePinRecovery(String pin) => _walletCore.completePinRecovery(pin);

  @override
  Future<void> cancelPinRecovery() => _walletCore.cancelPinRecovery();

  @override
  Future<WalletInstructionResult> changePin(String oldPin, String newPin) async {
    await _checkRegistration();
    return _walletCore.changePin(oldPin, newPin);
  }

  @override
  Future<WalletInstructionResult> continueChangePin(String pin) async {
    await _checkRegistration();
    return _walletCore.continueChangePin(pin);
  }

  /// Check if user has a registered wallet, used as a sanity check before performing certain pin related operations.
  Future<void> _checkRegistration() async {
    final isRegistered = await _walletCore.isRegistered();
    if (!isRegistered) throw StateError('Wallet not yet registered!');
  }
}
