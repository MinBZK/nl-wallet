import 'dart:async';

import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../pin_repository.dart';

class CorePinRepository extends PinRepository {
  final TypedWalletCore _walletCore;

  CorePinRepository(this._walletCore);

  @override
  Future<String> createPinRecoveryRedirectUri() => _walletCore.createPinRecoveryRedirectUri();

  @override
  Future<void> continuePinRecovery(String uri) => _walletCore.continuePinRecovery(uri);

  @override
  Future<void> completePinRecovery(String pin) => _walletCore.completePinRecovery(pin);

  @override
  Future<void> cancelPinRecovery() => _walletCore.cancelPinRecovery();
}
