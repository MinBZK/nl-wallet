import '../../../bridge_generated.dart';

abstract class TypedWalletCore {
  Future<PinValidationResult> isValidPin(String pin);

  Future<void> register(String pin);

  Future<bool> isRegistered();

  Future<void> lockWallet();

  Future<WalletInstructionResult> unlockWallet(String pin);

  Stream<bool> get isLocked;

  Future<String> createPidIssuanceRedirectUri();

  Future<IdentifyUriResult> identifyUri(String uri);

  Future<void> cancelPidIssuance();

  Stream<FlutterConfiguration> observeConfig();

  Future<WalletInstructionResult> acceptOfferedPid(String pin);

  Future<void> rejectOfferedPid();

  Stream<List<Card>> observeCards();

  Future<void> resetWallet();

  Future<List<Card>> continuePidIssuance(String uri);

  Future<StartDisclosureResult> startDisclosure(String uri);

  Future<void> cancelDisclosure();

  Future<WalletInstructionResult> acceptDisclosure(String pin);
}
