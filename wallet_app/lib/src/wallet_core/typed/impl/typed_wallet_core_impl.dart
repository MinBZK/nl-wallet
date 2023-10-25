import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:rxdart/rxdart.dart';

import '../../../util/mapper/mapper.dart';
import '../../error/core_error.dart';
import '../../error/flutter_api_error.dart';
import '../../typed/typed_wallet_core.dart';
import '../../wallet_core.dart';

/// Wraps the generated WalletCore.
/// Adds auto initialization, pass through of the locked
/// flag and parsing of the [FlutterApiError]s.
class TypedWalletCoreImpl extends TypedWalletCore {
  final WalletCore _walletCore;
  final Mapper<String, CoreError> _errorMapper;
  final Completer _isInitialized = Completer();
  final BehaviorSubject<bool> _isLocked = BehaviorSubject.seeded(true);
  final BehaviorSubject<FlutterConfiguration> _flutterConfig = BehaviorSubject();
  final BehaviorSubject<List<Card>> _cards = BehaviorSubject();

  TypedWalletCoreImpl(this._walletCore, this._errorMapper) {
    _initWalletCore();
    _setupLockedStream();
    _setupConfigurationStream();
    _setupCardsStream();
  }

  void _initWalletCore() async {
    if ((await _walletCore.isInitialized()) == false) {
      // Initialize the Asynchronous runtime and the wallet itself.
      // This is required to call any subsequent API function on the wallet.
      await _walletCore.init();
    } else {
      // The wallet_core is already initialized, this can happen when the Flutter
      // engine/activity was killed, but the application (and thus native code) was
      // kept alive by the platform. To recover from this we make sure the streams are reset,
      // as they can contain references to the previous Flutter engine.
      await _walletCore.clearLockStream();
      await _walletCore.clearConfigurationStream();
      await _walletCore.clearCardsStream();
      // Make sure the wallet is locked, as the [AutoLockObserver] was also killed.
      await _walletCore.lockWallet();
    }
    _isInitialized.complete();
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

  void _setupCardsStream() async {
    //FIXME: Ideally we don't set the card stream until we start observing it (i.e. in onListen())
    //FIXME: but since the cards are not persisted yet that means we might miss events, so observing
    //FIXME: the wallet_core cards stream through the complete lifecycle of the app for now.
    await _isInitialized.future;
    _walletCore.setCardsStream().listen((event) => _cards.add(event));
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
  Future<void> lockWallet() async {
    try {
      return await _walletCore.lockWallet();
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<WalletInstructionResult> unlockWallet(String pin) async {
    try {
      return await _walletCore.unlockWallet(pin: pin);
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Stream<bool> get isLocked => _isLocked;

  @override
  Stream<FlutterConfiguration> observeConfig() => _flutterConfig.stream;

  @override
  Future<String> createPidIssuanceRedirectUri() async {
    try {
      return await _walletCore.createPidIssuanceRedirectUri();
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<IdentifyUriResult> identifyUri(String uri) async {
    try {
      return await _walletCore.identifyUri(uri: uri);
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<void> cancelPidIssuance() async {
    try {
      return await _walletCore.cancelPidIssuance();
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<List<Card>> continuePidIssuance(Uri uri) async {
    try {
      return await _walletCore.continuePidIssuance(uri: uri.toString());
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<WalletInstructionResult> acceptOfferedPid(String pin) async {
    try {
      return await _walletCore.acceptPidIssuance(pin: pin);
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<void> rejectOfferedPid() async {
    try {
      return await _walletCore.rejectPidIssuance();
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<DisclosureResult> startDisclosure(Uri uri) async {
    try {
      return await _walletCore.startDisclosure(uri: uri.toString());
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<void> cancelDisclosure() async {
    try {
      return await _walletCore.cancelDisclosure();
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Future<WalletInstructionResult> acceptDisclosure(String pin) async {
    try {
      return await _walletCore.acceptDisclosure(pin: pin);
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  @override
  Stream<List<Card>> observeCards() => _cards.stream;

  @override
  Future<void> resetWallet() async {
    try {
      return await _walletCore.resetWallet();
    } catch (ex) {
      throw _handleCoreException(ex);
    }
  }

  /// Converts the exception to a [CoreError]
  /// if it can be mapped into one, otherwise returns
  /// the original exception.
  Object _handleCoreException(Object ex) {
    try {
      var coreErrorJson = '';
      if (ex is FrbAnyhowException) {
        Fimber.e('FrbAnyhowException contents: ${ex.anyhow}');
        coreErrorJson = ex.anyhow;
      } else if (ex is FfiException) {
        Fimber.e('FfiException contents. Code: ${ex.code}, Message: ${ex.message}');
        if (ex.code != 'RESULT_ERROR') return ex;
        coreErrorJson = ex.message;
      } else if (ex is String) {
        Fimber.e('String type exception. Contents: $ex');
        coreErrorJson = ex;
      }
      return _errorMapper.map(coreErrorJson);
    } catch (mapException) {
      Fimber.e('Failed to map exception to CoreError, returning original exception', ex: mapException);
      return ex;
    }
  }
}
