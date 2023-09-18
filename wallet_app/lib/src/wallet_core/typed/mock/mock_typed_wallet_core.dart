import 'dart:async';

import '../../wallet_core.dart';
import '../typed_wallet_core.dart';

// Simple STUB to satisfy di
class MockTypedWalletCore extends TypedWalletCore {
  @override
  Future<PinValidationResult> isValidPin(String pin) {
    // TODO: implement isValidPin
    throw UnimplementedError();
  }

  @override
  Future<void> register(String pin) {
    // TODO: implement register
    throw UnimplementedError();
  }

  @override
  Future<bool> isRegistered() {
    // TODO: implement isRegistered
    throw UnimplementedError();
  }

  @override
  Future<void> lockWallet() {
    // TODO: implement lockWallet
    throw UnimplementedError();
  }

  @override
  Future<WalletInstructionResult> unlockWallet(String pin) {
    // TODO: implement unlockWallet
    throw UnimplementedError();
  }

  @override
  // TODO: implement isLocked
  Stream<bool> get isLocked => throw UnimplementedError();

  @override
  Future<String> createPidIssuanceRedirectUri() {
    // TODO: implement createPidIssuanceRedirectUri
    throw UnimplementedError();
  }

  @override
  Stream<ProcessUriEvent> processUri(Uri uri) {
    // TODO: implement processUri
    throw UnimplementedError();
  }

  @override
  Future<void> cancelPidIssuance() {
    // TODO: implement cancelPidIssuance
    throw UnimplementedError();
  }

  @override
  Stream<FlutterConfiguration> observeConfig() {
    // TODO: implement observeConfig
    throw UnimplementedError();
  }

  @override
  Future<WalletInstructionResult> acceptOfferedPid(String pin) {
    // TODO: implement acceptOfferedPid
    throw UnimplementedError();
  }

  @override
  Future<void> rejectOfferedPid() {
    // TODO: implement rejectOfferedPid
    throw UnimplementedError();
  }

  @override
  Stream<List<Card>> observeCards() {
    // TODO: implement observeCards
    throw UnimplementedError();
  }
}
