import 'dart:async';
import 'dart:convert';

import 'package:fimber/fimber.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:rxdart/rxdart.dart';

import '../error/flutter_api_error.dart';
import '../typed_wallet_core.dart';
import '../wallet_core.dart';

/// Wraps the generated WalletCore.
/// Adds auto initialization, pass through of the locked
/// flag and parsing of the [FlutterApiError]s.
class TypedWalletCoreImpl extends TypedWalletCore {
  final WalletCore _walletCore;
  final BehaviorSubject<bool> _isLocked = BehaviorSubject.seeded(true);

  TypedWalletCoreImpl(this._walletCore) {
    // Initialize the Asynchronous runtime and the wallet itself.
    // This is required to call any subsequent API function on the wallet.
    _walletCore.init().listen(
      (locked) => _isLocked.add(locked),
      onError: (ex) {
        Fimber.e('WalletCore failed to initialize!', ex: ex);
        throw ex; //Delegate to [WalletErrorHandler]
      },
    );
  }

  @override
  Future<PinValidationResult> isValidPin(String pin) async {
    try {
      return await _walletCore.isValidPin(pin: pin);
    } catch (ex) {
      _decodeFlutterApiError(ex);
      rethrow;
    }
  }

  @override
  Future<void> register(String pin) async {
    try {
      return await _walletCore.register(pin: pin);
    } catch (ex) {
      _decodeFlutterApiError(ex);
      rethrow;
    }
  }

  @override
  Future<bool> isRegistered() async {
    try {
      return await _walletCore.hasRegistration();
    } catch (ex) {
      _decodeFlutterApiError(ex);
      rethrow;
    }
  }

  @override
  Future<String> getDigidAuthUrl() async {
    try {
      return await _walletCore.getDigidAuthUrl();
    } catch (ex) {
      _decodeFlutterApiError(ex);
      rethrow;
    }
  }

  @override
  Future<void> lockWallet() async {
    try {
      return await _walletCore.lockWallet();
    } catch (ex) {
      _decodeFlutterApiError(ex);
      rethrow;
    }
  }

  @override
  Future<WalletUnlockResult> unlockWallet(String pin) async {
    try {
      return await _walletCore.unlockWallet(pin: pin);
    } catch (ex) {
      _decodeFlutterApiError(ex);
      rethrow;
    }
  }

  @override
  Stream<bool> get isLocked => _isLocked;

  @override
  Stream<UriFlowEvent> processUri(Uri uri) {
    try {
      return _walletCore.processUri(uri: uri.toString());
    } catch (ex) {
      _decodeFlutterApiError(ex);
      rethrow;
    }
  }

  /// Check the exception and throws it as a [FlutterApiError]
  /// if it can be mapped into one.
  void _decodeFlutterApiError(Object ex) {
    if (ex is FfiException) {
      if (ex.code != 'RESULT_ERROR') return;
      var decodedJson = json.decode(ex.message);
      throw FlutterApiError.fromJson(decodedJson);
    }
  }
}
