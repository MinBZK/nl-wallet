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
  final Completer _isInitialized = Completer();
  final BehaviorSubject<bool> _isLocked = BehaviorSubject.seeded(true);
  final BehaviorSubject<FlutterConfiguration> _flutterConfig = BehaviorSubject();

  TypedWalletCoreImpl(this._walletCore) {
    // Initialize the Asynchronous runtime and the wallet itself.
    // This is required to call any subsequent API function on the wallet.
    _walletCore.init().then(
      (value) => _isInitialized.complete(),
      onError: (ex) {
        Fimber.e('WalletCore failed to initialize!', ex: ex);
        throw ex; //Delegate to [WalletErrorHandler]
      },
    );

    _setupLockedStream();
    _setupConfigurationStream();
  }

  void _setupLockedStream() {
    _isLocked.onListen = () async {
      await _isInitialized.future;
      _walletCore.setLockStream().listen((event) => _isLocked.add(event));
    };
    _isLocked.onCancel = () => _walletCore.clearLockStream();
  }

  void _setupConfigurationStream() {
    _flutterConfig.onListen = () async {
      await _isInitialized.future;
      _walletCore.setConfigurationStream().listen((event) => _flutterConfig.add(event));
    };
    _flutterConfig.onCancel = () => _walletCore.clearConfigurationStream();
  }

  @override
  Future<PinValidationResult> isValidPin(String pin) async {
    try {
      return await _walletCore.isValidPin(pin: pin);
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<void> register(String pin) async {
    try {
      return await _walletCore.register(pin: pin);
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<bool> isRegistered() async {
    try {
      return await _walletCore.hasRegistration();
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<String> getDigidAuthUrl() async {
    try {
      return await _walletCore.getDigidAuthUrl();
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<void> lockWallet() async {
    try {
      return await _walletCore.lockWallet();
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<WalletUnlockResult> unlockWallet(String pin) async {
    try {
      return await _walletCore.unlockWallet(pin: pin);
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Stream<bool> get isLocked => _isLocked;

  @override
  Stream<UriFlowEvent> processUri(Uri uri) {
    try {
      return _walletCore.processUri(uri: uri.toString());
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  /// Converts the exception to a [FlutterApiError]
  /// if it can be mapped into one, otherwise returns
  /// the original exception.
  Object _handleCoreException(Object ex) {
    if (ex is FfiException) {
      if (ex.code != 'RESULT_ERROR') return ex;
      var decodedJson = json.decode(ex.message);
      return FlutterApiError.fromJson(decodedJson);
    } else {
      return ex;
    }
  }

  @override
  Stream<FlutterConfiguration> observeConfig() => _flutterConfig.stream;
}
