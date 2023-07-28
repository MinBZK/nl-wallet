import 'dart:async';

import '../typed_wallet_core.dart';
import '../wallet_core.dart';

// Simple STUB to satisfy di
class MockWalletCore extends TypedWalletCore {
  @override
  Future<String> getDigidAuthUrl() {
    // TODO: implement getDigidAuthUrl
    throw UnimplementedError();
  }

  @override
  // TODO: implement isLocked
  Stream<bool> get isLocked => throw UnimplementedError();

  @override
  Future<bool> isRegistered() {
    // TODO: implement isRegistered
    throw UnimplementedError();
  }

  @override
  Future<PinValidationResult> isValidPin(String pin) {
    // TODO: implement isValidPin
    throw UnimplementedError();
  }

  @override
  Future<void> lockWallet() {
    // TODO: implement lockWallet
    throw UnimplementedError();
  }

  @override
  Stream<FlutterConfiguration> observeConfig() {
    // TODO: implement observeConfig
    throw UnimplementedError();
  }

  @override
  Stream<UriFlowEvent> processUri(Uri uri) {
    // TODO: implement processUri
    throw UnimplementedError();
  }

  @override
  Future<void> register(String pin) {
    // TODO: implement register
    throw UnimplementedError();
  }

  @override
  Future<WalletUnlockResult> unlockWallet(String pin) {
    // TODO: implement unlockWallet
    throw UnimplementedError();
  }
}
