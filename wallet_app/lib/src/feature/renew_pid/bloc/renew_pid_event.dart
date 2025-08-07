part of 'renew_pid_bloc.dart';

abstract class RenewPidEvent extends Equatable {
  const RenewPidEvent();

  @override
  List<Object?> get props => [];
}

class RenewPidLoginWithDigidClicked extends RenewPidEvent {
  const RenewPidLoginWithDigidClicked();
}

class RenewPidContinuePidRenewal extends RenewPidEvent {
  final String authUrl;

  const RenewPidContinuePidRenewal(this.authUrl);

  @override
  List<Object?> get props => [authUrl];
}

class RenewPidAttributesRejected extends RenewPidEvent {
  const RenewPidAttributesRejected();
}

class RenewPidAttributesConfirmed extends RenewPidEvent {
  final List<Attribute> previewAttributes;

  const RenewPidAttributesConfirmed(this.previewAttributes);

  @override
  List<Object?> get props => [previewAttributes];
}

class RenewPidPinConfirmed extends RenewPidEvent {}

class RenewPidPinConfirmationFailed extends RenewPidEvent {
  final ApplicationError error;

  const RenewPidPinConfirmationFailed({required this.error});

  @override
  List<Object?> get props => [error];
}

class RenewPidLaunchDigidUrlFailed extends RenewPidEvent {
  final ApplicationError error;

  const RenewPidLaunchDigidUrlFailed({required this.error});

  @override
  List<Object?> get props => [error];
}

class RenewPidLoginWithDigidFailed extends RenewPidEvent {
  final ApplicationError error;

  final bool cancelledByUser;

  const RenewPidLoginWithDigidFailed({required this.error, this.cancelledByUser = false});

  @override
  List<Object?> get props => [error, cancelledByUser];
}

class RenewPidRetryPressed extends RenewPidEvent {
  const RenewPidRetryPressed();
}

class RenewPidStopPressed extends RenewPidEvent {
  const RenewPidStopPressed();
}

class RenewPidBackPressed extends RenewPidEvent {
  const RenewPidBackPressed();
}
