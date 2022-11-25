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

  const WalletPersonalizeOfferingAccepted(this.acceptedCard);

  @override
  List<Object?> get props => [acceptedCard, ...super.props];
}

class WalletPersonalizeOnRetryClicked extends WalletPersonalizeEvent {}
