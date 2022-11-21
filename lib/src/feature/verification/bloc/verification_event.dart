part of 'verification_bloc.dart';

abstract class VerificationEvent extends Equatable {
  const VerificationEvent();
}

class VerificationLoadRequested extends VerificationEvent {
  final String sessionId;

  const VerificationLoadRequested(this.sessionId);

  @override
  List<Object?> get props => [sessionId];
}

class VerificationOrganizationApproved extends VerificationEvent {
  const VerificationOrganizationApproved();

  @override
  List<Object?> get props => [];
}

class VerificationShareRequestedAttributesApproved extends VerificationEvent {
  const VerificationShareRequestedAttributesApproved();

  @override
  List<Object?> get props => [];
}

class VerificationPinConfirmed extends VerificationEvent {
  const VerificationPinConfirmed();

  @override
  List<Object?> get props => [];
}

class VerificationBackPressed extends VerificationEvent {
  const VerificationBackPressed();

  @override
  List<Object?> get props => [];
}

class VerificationStopRequested extends VerificationEvent {
  const VerificationStopRequested();

  @override
  List<Object?> get props => [];
}
