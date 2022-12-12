part of 'issuance_bloc.dart';

abstract class IssuanceEvent extends Equatable {
  const IssuanceEvent();
}

class IssuanceLoadTriggered extends IssuanceEvent {
  final String sessionId;
  final bool isRefreshFlow;

  const IssuanceLoadTriggered(this.sessionId, this.isRefreshFlow);

  @override
  List<Object?> get props => [sessionId, isRefreshFlow];
}

class IssuanceOrganizationApproved extends IssuanceEvent {
  const IssuanceOrganizationApproved();

  @override
  List<Object?> get props => [];
}

class IssuanceBackPressed extends IssuanceEvent {
  const IssuanceBackPressed();

  @override
  List<Object?> get props => [];
}

class IssuanceOrganizationDeclined extends IssuanceEvent {
  const IssuanceOrganizationDeclined();

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

class IssuanceStopRequested extends IssuanceEvent {
  final IssuanceFlow? flow;

  const IssuanceStopRequested(this.flow);

  @override
  List<Object?> get props => [flow];
}
