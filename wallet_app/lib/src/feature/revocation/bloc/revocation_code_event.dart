part of 'revocation_code_bloc.dart';

abstract class RevocationCodeEvent extends Equatable {
  const RevocationCodeEvent();

  @override
  List<Object> get props => [];
}

class RevocationCodeLoadTriggered extends RevocationCodeEvent {
  const RevocationCodeLoadTriggered();
}

class RevocationCodeContinuePressed extends RevocationCodeEvent {
  const RevocationCodeContinuePressed();
}
