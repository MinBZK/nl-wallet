part of 'sign_bloc.dart';

abstract class SignEvent extends Equatable {
  const SignEvent();

  @override
  List<Object?> get props => [];
}

class SignLoadTriggered extends SignEvent {
  final String id;

  const SignLoadTriggered(this.id);

  @override
  List<Object?> get props => [id];
}

class SignBackPressed extends SignEvent {
  const SignBackPressed();
}

class SignOrganizationApproved extends SignEvent {
  const SignOrganizationApproved();
}

class SignAgreementChecked extends SignEvent {
  const SignAgreementChecked();
}

class SignAgreementApproved extends SignEvent {
  const SignAgreementApproved();
}

class SignPinConfirmed extends SignEvent {
  const SignPinConfirmed();
}

class SignStopRequested extends SignEvent {
  const SignStopRequested();
}
