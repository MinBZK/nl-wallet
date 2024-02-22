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
}

class WalletPersonalizeAuthInProgress extends WalletPersonalizeEvent {}

class WalletPersonalizeLoginWithDigidSucceeded extends WalletPersonalizeEvent {
  final List<Attribute> previewAttributes;

  const WalletPersonalizeLoginWithDigidSucceeded(this.previewAttributes);
}

class WalletPersonalizeLoginWithDigidFailed extends WalletPersonalizeEvent {
  final bool cancelledByUser;

  const WalletPersonalizeLoginWithDigidFailed({this.cancelledByUser = false});
}

class WalletPersonalizeOfferingAccepted extends WalletPersonalizeEvent {
  final List<Attribute> previewAttributes;

  const WalletPersonalizeOfferingAccepted(this.previewAttributes);
}

class WalletPersonalizeOfferingRejected extends WalletPersonalizeEvent {}

class WalletPersonalizeRetryPressed extends WalletPersonalizeEvent {}

class WalletPersonalizeBackPressed extends WalletPersonalizeEvent {}

class WalletPersonalizePinConfirmed extends WalletPersonalizeEvent {}
