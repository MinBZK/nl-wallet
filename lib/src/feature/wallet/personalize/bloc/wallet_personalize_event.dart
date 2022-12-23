part of 'wallet_personalize_bloc.dart';

abstract class WalletPersonalizeEvent extends Equatable {
  const WalletPersonalizeEvent();

  @override
  List<Object?> get props => [];
}

class WalletPersonalizeLoginWithDigidClicked extends WalletPersonalizeEvent {}

class WalletPersonalizeLoginWithDigidSucceeded extends WalletPersonalizeEvent {}

class WalletPersonalizeOfferingAccepted extends WalletPersonalizeEvent {
  final WalletCard acceptedCard;
  final Organization issuingOrganization;

  const WalletPersonalizeOfferingAccepted(this.acceptedCard, this.issuingOrganization);

  @override
  List<Object?> get props => [acceptedCard, issuingOrganization, ...super.props];
}

class WalletPersonalizeOfferingVerified extends WalletPersonalizeEvent {}

class WalletPersonalizeScanInitiated extends WalletPersonalizeEvent {}

class WalletPersonalizeScanEvent extends WalletPersonalizeEvent {}

class WalletPersonalizePhotoApproved extends WalletPersonalizeEvent {}

class WalletPersonalizeOnRetryClicked extends WalletPersonalizeEvent {}

class WalletPersonalizeOnBackPressed extends WalletPersonalizeEvent {}

class WalletPersonalizeRetrieveMoreCardsPressed extends WalletPersonalizeEvent {}

class WalletPersonalizeSkipRetrieveMoreCardsPressed extends WalletPersonalizeEvent {}

class WalletPersonalizeSkipAddMoreCardsPressed extends WalletPersonalizeEvent {}

class WalletPersonalizeAddSelectedCardsPressed extends WalletPersonalizeEvent {}

class WalletPersonalizeDataOnCardDeclined extends WalletPersonalizeEvent {}

class WalletPersonalizeDataOnCardConfirmed extends WalletPersonalizeEvent {}

class WalletPersonalizePinConfirmed extends WalletPersonalizeEvent {}

class WalletPersonalizeSelectedCardToggled extends WalletPersonalizeEvent {
  final WalletCard card;

  const WalletPersonalizeSelectedCardToggled(this.card);
}
