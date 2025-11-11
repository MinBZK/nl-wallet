import 'package:equatable/equatable.dart';

sealed class CheckPinResult extends Equatable {}

class CheckPinResultIncorrect extends CheckPinResult {
  final int attemptsLeftInRound;
  final bool isFinalRound;

  CheckPinResultIncorrect({
    required this.attemptsLeftInRound,
    this.isFinalRound = false,
  });

  @override
  List<Object?> get props => [attemptsLeftInRound, isFinalRound];
}

class CheckPinResultTimeout extends CheckPinResult {
  final int timeoutMillis;

  CheckPinResultTimeout({required this.timeoutMillis});

  @override
  List<Object?> get props => [timeoutMillis];
}

class CheckPinResultBlocked extends CheckPinResult {
  @override
  List<Object?> get props => [];
}
