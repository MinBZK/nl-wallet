import '../../../wallet_constants.dart';

class GetAvailablePinAttemptsUseCase {
  var _attemptsLeft = kPinAttempts;

  GetAvailablePinAttemptsUseCase();

  Future<int> getLeftoverAttempts() async {
    _attemptsLeft--;
    return _attemptsLeft;
  }

  /// Convenience method to reset the counter while mocking. Eventually
  /// this value should be persisted securely somewhere and only been reset
  /// after destroying the wallet.
  void reset() => _attemptsLeft = kPinAttempts;
}
