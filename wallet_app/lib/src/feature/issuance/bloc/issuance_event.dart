part of 'issuance_bloc.dart';

abstract class IssuanceEvent extends Equatable {
  const IssuanceEvent();
}

class IssuanceOrganizationApproved extends IssuanceEvent {
  const IssuanceOrganizationApproved();

  @override
  List<Object?> get props => [];
}

class IssuanceInitiated extends IssuanceEvent {
  final String issuanceUri;

  const IssuanceInitiated(this.issuanceUri);

  @override
  List<Object?> get props => [issuanceUri];
}

class IssuanceBackPressed extends IssuanceEvent {
  const IssuanceBackPressed();

  @override
  List<Object?> get props => [];
}

class IssuanceShareRequestedAttributesApproved extends IssuanceEvent {
  const IssuanceShareRequestedAttributesApproved();

  @override
  List<Object?> get props => [];
}

class IssuanceShareRequestedAttributesDeclined extends IssuanceEvent {
  const IssuanceShareRequestedAttributesDeclined();

  @override
  List<Object?> get props => [];
}

class IssuancePinConfirmed extends IssuanceEvent {
  const IssuancePinConfirmed();

  @override
  List<Object?> get props => [];
}

class IssuanceCheckDataOfferingApproved extends IssuanceEvent {
  const IssuanceCheckDataOfferingApproved();

  @override
  List<Object?> get props => [];
}

class IssuanceCardToggled extends IssuanceEvent {
  final WalletCard card;

  const IssuanceCardToggled(this.card);

  @override
  List<Object?> get props => [card];
}

class IssuanceSelectedCardsConfirmed extends IssuanceEvent {
  const IssuanceSelectedCardsConfirmed();

  @override
  List<Object?> get props => [];
}

class IssuanceCardDeclined extends IssuanceEvent {
  final WalletCard card;

  const IssuanceCardDeclined(this.card);

  @override
  List<Object?> get props => [card];
}

class IssuanceCardApproved extends IssuanceEvent {
  final WalletCard card;

  const IssuanceCardApproved(this.card);

  @override
  List<Object?> get props => [card];
}

class IssuanceStopRequested extends IssuanceEvent {
  const IssuanceStopRequested();

  @override
  List<Object?> get props => [];
}

class IssuanceUpdateState extends IssuanceEvent {
  final IssuanceState state;

  const IssuanceUpdateState(this.state);

  @override
  List<Object?> get props => [state];
}
