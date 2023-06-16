abstract class CheckPinResult {}

class CheckPinResultOk extends CheckPinResult {}

class CheckPinResultIncorrect extends CheckPinResult {
  final int leftoverAttempts;
  final bool isFinalAttempt;

  CheckPinResultIncorrect({required this.leftoverAttempts, this.isFinalAttempt = false});
}

class CheckPinResultTimeout extends CheckPinResult {
  final int timeoutMillis;

  CheckPinResultTimeout({required this.timeoutMillis});
}

class CheckPinResultBlocked extends CheckPinResult {}

class CheckPinResultGenericError extends CheckPinResult {}