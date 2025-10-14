import 'package:wallet_core/core.dart';

abstract class PinRepository {
  /// Validates the supplied [pin]
  ///
  /// Throws a [PinValidationError] if the pin does
  /// not meet the required standards.
  Future<void> validatePin(String pin);

  /// Check if the provided pin matches the one that is registered
  Future<WalletInstructionResult> checkPin(String pin);

  /// Creates a DigiD redirect URI to start the PIN recovery process.
  Future<String> createPinRecoveryRedirectUri();

  /// Continues the PIN recovery process with the given [uri] (obtained by completing the DigiD login).
  Future<void> continuePinRecovery(String uri);

  /// Completes the PIN recovery process and saves the new [pin].
  Future<void> completePinRecovery(String pin);

  /// Cancels the current PIN recovery process.
  Future<void> cancelPinRecovery();

  /// Request a pin change from [oldPin] to [newPin]. Only succeeds when [oldPin] matches the current pin and [newPin]
  /// is considered valid. NOTE: This should be followed up by a call to [continueChangePin] when this call succeeds
  /// to immediately confirm the pin change. When the call errors out, it's good practice (but not immediately mandatory)
  /// to call [continueChangePin] with the old pin. This 'commiting' [continueChangePin] otherwise happens silently inside
  /// wallet_core whenever the device unlocks/confirms with a pin.
  Future<WalletInstructionResult> changePin(String oldPin, String newPin);

  /// Confirm the pin change, should be called after [changePin] returns success
  Future<WalletInstructionResult> continueChangePin(String pin);
}
