import 'package:wallet_core/core.dart';

class WalletCoreMock extends _FlutterRustBridgeTasksMeta implements WalletCore {
  bool _isInitialized = false;

  WalletCoreMock();

  @override
  Future<WalletInstructionResult> acceptDisclosure({required String pin, hint}) {
    // TODO: implement acceptDisclosure
    throw UnimplementedError();
  }

  @override
  Future<WalletInstructionResult> acceptPidIssuance({required String pin, hint}) {
    // TODO: implement acceptPidIssuance
    throw UnimplementedError();
  }

  @override
  Future<void> cancelDisclosure({hint}) {
    // TODO: implement cancelDisclosure
    throw UnimplementedError();
  }

  @override
  Future<void> cancelPidIssuance({hint}) {
    // TODO: implement cancelPidIssuance
    throw UnimplementedError();
  }

  @override
  Future<void> clearCardsStream({hint}) {
    // TODO: implement clearCardsStream
    throw UnimplementedError();
  }

  @override
  Future<void> clearConfigurationStream({hint}) {
    // TODO: implement clearConfigurationStream
    throw UnimplementedError();
  }

  @override
  Future<void> clearLockStream({hint}) {
    // TODO: implement clearLockStream
    throw UnimplementedError();
  }

  @override
  Future<List<Card>> continuePidIssuance({required String uri, hint}) {
    // TODO: implement continuePidIssuance
    throw UnimplementedError();
  }

  @override
  Future<String> createPidIssuanceRedirectUri({hint}) {
    // TODO: implement createPidIssuanceRedirectUri
    throw UnimplementedError();
  }

  @override
  Future<bool> hasRegistration({hint}) {
    // TODO: implement hasRegistration
    throw UnimplementedError();
  }

  @override
  Future<IdentifyUriResult> identifyUri({required String uri, hint}) {
    // TODO: implement identifyUri
    throw UnimplementedError();
  }

  @override
  Future<void> init({hint}) async {
    _isInitialized = true;
  }

  @override
  Future<bool> isInitialized({hint}) async => _isInitialized;

  @override
  Future<PinValidationResult> isValidPin({required String pin, hint}) {
    // TODO: implement isValidPin
    throw UnimplementedError();
  }

  @override
  Future<void> lockWallet({hint}) {
    // TODO: implement lockWallet
    throw UnimplementedError();
  }

  @override
  Future<void> register({required String pin, hint}) {
    // TODO: implement register
    throw UnimplementedError();
  }

  @override
  Future<void> rejectPidIssuance({hint}) {
    // TODO: implement rejectPidIssuance
    throw UnimplementedError();
  }

  @override
  Future<void> resetWallet({hint}) {
    // TODO: implement resetWallet
    throw UnimplementedError();
  }

  @override
  Stream<List<Card>> setCardsStream({hint}) {
    // TODO: implement setCardsStream
    throw UnimplementedError();
  }

  @override
  Stream<FlutterConfiguration> setConfigurationStream({hint}) {
    // TODO: implement setConfigurationStream
    throw UnimplementedError();
  }

  @override
  Stream<bool> setLockStream({hint}) {
    // TODO: implement setLockStream
    throw UnimplementedError();
  }

  @override
  Future<StartDisclosureResult> startDisclosure({required String uri, hint}) {
    // TODO: implement startDisclosure
    throw UnimplementedError();
  }

  @override
  Future<WalletInstructionResult> unlockWallet({required String pin, hint}) {
    // TODO: implement unlockWallet
    throw UnimplementedError();
  }

  @override
  Future<List<WalletEvent>> getHistory({hint}) {
    // TODO: implement getHistory
    throw UnimplementedError();
  }

  @override
  Future<List<WalletEvent>> getHistoryForCard({required String docType, hint}) {
    // TODO: implement getHistoryForCard
    throw UnimplementedError();
  }

  @override
  // TODO: implement kGetHistoryConstMeta
  FlutterRustBridgeTaskConstMeta get kGetHistoryConstMeta => throw UnimplementedError();

  @override
  // TODO: implement kGetHistoryForCardConstMeta
  FlutterRustBridgeTaskConstMeta get kGetHistoryForCardConstMeta => throw UnimplementedError();
}

/// Helper class to make [WalletCoreMock] satisfy [WalletCore]
/// without cluttering it with getters we don't intend to implement.
class _FlutterRustBridgeTasksMeta {
  FlutterRustBridgeTaskConstMeta get kAcceptDisclosureConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kAcceptPidIssuanceConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kCancelDisclosureConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kCancelPidIssuanceConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kClearCardsStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kClearConfigurationStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kClearLockStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kContinuePidIssuanceConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kCreatePidIssuanceRedirectUriConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kHasRegistrationConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kIdentifyUriConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kInitConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kIsInitializedConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kIsValidPinConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kLockWalletConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kRegisterConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kRejectPidIssuanceConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kResetWalletConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kSetCardsStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kSetConfigurationStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kSetLockStreamConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kStartDisclosureConstMeta => throw UnimplementedError();

  FlutterRustBridgeTaskConstMeta get kUnlockWalletConstMeta => throw UnimplementedError();
}
