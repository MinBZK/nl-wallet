part of 'issuance_bloc.dart';

abstract class IssuanceEvent extends Equatable {
  const IssuanceEvent();

  @override
  List<Object?> get props => [];
}

class IssuanceOrganizationApproved extends IssuanceEvent {
  const IssuanceOrganizationApproved();
}

class IssuanceSessionStarted extends IssuanceEvent {
  final String issuanceUri;
  final bool isQrCode;

  const IssuanceSessionStarted(this.issuanceUri, {this.isQrCode = false});

  @override
  List<Object?> get props => [issuanceUri, isQrCode];
}

class IssuanceBackPressed extends IssuanceEvent {
  const IssuanceBackPressed();
}

class IssuanceShareRequestedAttributesDeclined extends IssuanceEvent {
  const IssuanceShareRequestedAttributesDeclined();
}

class IssuancePinForDisclosureConfirmed extends IssuanceEvent {
  final List<WalletCard> cards;

  const IssuancePinForDisclosureConfirmed({required this.cards});

  @override
  List<Object?> get props => [cards];
}

class IssuancePinForIssuanceConfirmed extends IssuanceEvent {
  const IssuancePinForIssuanceConfirmed();
}

class IssuanceApproveCards extends IssuanceEvent {
  final List<WalletCard> cards;

  const IssuanceApproveCards({required this.cards});

  @override
  List<Object?> get props => [cards];
}

class IssuanceCardToggled extends IssuanceEvent {
  final WalletCard card;

  const IssuanceCardToggled(this.card);

  @override
  List<Object?> get props => [card];
}

class IssuanceStopRequested extends IssuanceEvent {
  const IssuanceStopRequested();
}

class IssuanceConfirmPinFailed extends IssuanceEvent {
  final ApplicationError error;

  const IssuanceConfirmPinFailed({required this.error});

  @override
  List<Object?> get props => [error];
}

class IssuanceAlternativeCardSelected extends IssuanceEvent {
  final DiscloseCardRequest updatedCardRequest;

  const IssuanceAlternativeCardSelected(this.updatedCardRequest);

  @override
  List<Object?> get props => [updatedCardRequest];
}
