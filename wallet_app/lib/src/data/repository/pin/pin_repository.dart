abstract class PinRepository {
  Future<String> createPinRecoveryRedirectUri();

  Future<void> continuePinRecovery(String uri);

  Future<void> completePinRecovery(String pin);

  Future<void> cancelPinRecovery();
}
