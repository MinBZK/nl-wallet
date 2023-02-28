abstract class CheckIsValidPinUseCase {
  /// Validates the supplied [pin]
  ///
  /// Throws a [PinValidationError] if the pin does
  /// not meet the required standards.
  Future<void> invoke(String pin);
}
