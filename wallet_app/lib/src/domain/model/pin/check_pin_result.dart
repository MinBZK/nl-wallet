sealed class CheckPinResult {}

class CheckPinResultOk extends CheckPinResult {
  final String? returnUrl;

  CheckPinResultOk({this.returnUrl});
}

class CheckPinResultIncorrect extends CheckPinResult {
  final int attemptsLeftInRound;
  final bool isFinalRound;

  CheckPinResultIncorrect({
    required this.attemptsLeftInRound,
    this.isFinalRound = false,
  });
}

class CheckPinResultTimeout extends CheckPinResult {
  final int timeoutMillis;

  CheckPinResultTimeout({required this.timeoutMillis});
}

class CheckPinResultBlocked extends CheckPinResult {}
