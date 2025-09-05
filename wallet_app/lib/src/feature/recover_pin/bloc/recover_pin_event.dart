part of 'recover_pin_bloc.dart';

abstract class RecoverPinEvent extends Equatable {
  const RecoverPinEvent();

  @override
  List<Object?> get props => [];
}

class RecoverPinLoginWithDigidClicked extends RecoverPinEvent {
  const RecoverPinLoginWithDigidClicked();
}

class RecoverPinContinuePinRecovery extends RecoverPinEvent {
  final String authUrl;

  const RecoverPinContinuePinRecovery(this.authUrl);

  @override
  List<Object?> get props => [authUrl];
}

class RecoverPinDigitPressed extends RecoverPinEvent {
  final int digit;

  const RecoverPinDigitPressed(this.digit);

  @override
  List<Object?> get props => [digit];
}

class RecoverPinBackspacePressed extends RecoverPinEvent {}

class RecoverPinClearPressed extends RecoverPinEvent {}

class RecoverPinNewPinConfirmed extends RecoverPinEvent {
  final String pin;
  final String authUrl;

  const RecoverPinNewPinConfirmed({required this.pin, required this.authUrl});

  @override
  List<Object?> get props => [pin, authUrl, ...super.props];
}

class RecoverPinLaunchDigidUrlFailed extends RecoverPinEvent {
  final ApplicationError error;

  const RecoverPinLaunchDigidUrlFailed({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class RecoverPinLoginWithDigidFailed extends RecoverPinEvent {
  final ApplicationError error;

  final bool cancelledByUser;

  const RecoverPinLoginWithDigidFailed({required this.error, this.cancelledByUser = false});

  @override
  List<Object?> get props => [error, cancelledByUser, ...super.props];
}

class RecoverPinRetryPressed extends RecoverPinEvent {
  const RecoverPinRetryPressed();
}

class RecoverPinStopPressed extends RecoverPinEvent {
  const RecoverPinStopPressed();
}

class RecoverPinBackPressed extends RecoverPinEvent {
  const RecoverPinBackPressed();
}
