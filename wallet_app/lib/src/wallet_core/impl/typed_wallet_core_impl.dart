import 'dart:async';

import 'package:core_domain/core_domain.dart';
import 'package:fimber/fimber.dart';

import '../typed_wallet_core.dart';
import '../wallet_core.dart';

/// Wraps the generated WalletCore to provide
/// a typed interface using the SERDE generated
/// models from the 'core_domain' package.
class TypedWalletCoreImpl extends TypedWalletCore {
  final WalletCore _walletCore;

  TypedWalletCoreImpl(this._walletCore) {
    // Initialize the Asynchronous runtime and the wallet itself.
    // This is required to call any subsequent API function on the wallet.
    _walletCore.init().then(
      (_) => Fimber.d('WalletCore initialized!'),
      onError: (ex) {
        Fimber.e('WalletCore failed to initialize!', ex: ex);
        throw ex; //Delegate to [WalletErrorHandler]
      },
    );
  }

  @override
  Future<PinValidationResult> isValidPin(String pin) async {
    final bytes = await _walletCore.isValidPin(pin: pin);
    return PinValidationResultExtension.bincodeDeserialize(bytes);
  }

  @override
  Future<void> register(String pin) => _walletCore.register(pin: pin);

  @override
  Future<bool> isRegistered() => _walletCore.hasRegistration();

  @override
  Future<String> getDigidAuthUrl() => _walletCore.getDigidAuthUrl();

  @override
  Stream<UriFlowEvent> processUri(Uri uri) {
    return _walletCore.processUri(uri: uri.toString()).map((bytes) {
      return UriFlowEvent.bincodeDeserialize(bytes);
    });
  }
}
