part of 'wallet_personalize_bloc.dart';

abstract class WalletPersonalizeEvent extends Equatable {
  const WalletPersonalizeEvent();

  @override
  List<Object?> get props => [];
}

class WalletPersonalizeLoginWithDigidClicked extends WalletPersonalizeEvent {}

class WalletPersonalizeUpdateState extends WalletPersonalizeEvent {
  final WalletPersonalizeState state;

  const WalletPersonalizeUpdateState(this.state);

  @override
  List<Object?> get props => [state, ...super.props];
}

class WalletPersonalizeContinuePidIssuance extends WalletPersonalizeEvent {
  final String authUrl;

  const WalletPersonalizeContinuePidIssuance(this.authUrl);

  @override
  List<Object?> get props => [authUrl, ...super.props];
}

class WalletPersonalizeLoginWithDigidSucceeded extends WalletPersonalizeEvent {
  final List<Attribute> previewAttributes;

  const WalletPersonalizeLoginWithDigidSucceeded(this.previewAttributes);

  @override
  List<Object?> get props => [previewAttributes, ...super.props];
}

class WalletPersonalizeLoginWithDigidFailed extends WalletPersonalizeEvent {
  final ApplicationError error;

  final bool cancelledByUser;

  const WalletPersonalizeLoginWithDigidFailed({required this.error, this.cancelledByUser = false});

  @override
  List<Object?> get props => [error, cancelledByUser, ...super.props];
}

class WalletPersonalizeAcceptPidFailed extends WalletPersonalizeEvent {
  final ApplicationError error;

  const WalletPersonalizeAcceptPidFailed({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class WalletPersonalizeOfferingAccepted extends WalletPersonalizeEvent {
  final List<Attribute> previewAttributes;

  const WalletPersonalizeOfferingAccepted(this.previewAttributes);

  @override
  List<Object?> get props => [previewAttributes, ...super.props];
}

class WalletPersonalizeOfferingRejected extends WalletPersonalizeEvent {}

class WalletPersonalizeRetryPressed extends WalletPersonalizeEvent {}

class WalletPersonalizeBackPressed extends WalletPersonalizeEvent {}

class WalletPersonalizePinConfirmed extends WalletPersonalizeEvent {
  final TransferState transferState;

  const WalletPersonalizePinConfirmed(this.transferState);

  @override
  List<Object?> get props => [transferState, ...super.props];
}
