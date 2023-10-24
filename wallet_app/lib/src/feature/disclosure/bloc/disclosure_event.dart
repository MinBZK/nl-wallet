part of 'disclosure_bloc.dart';

abstract class DisclosureEvent extends Equatable {
  const DisclosureEvent();
}

class DisclosureLoadRequested extends DisclosureEvent {
  final String sessionId;

  const DisclosureLoadRequested(this.sessionId);

  @override
  List<Object?> get props => [sessionId];
}

class DisclosureOrganizationApproved extends DisclosureEvent {
  const DisclosureOrganizationApproved();

  @override
  List<Object?> get props => [];
}

class DisclosureShareRequestedAttributesApproved extends DisclosureEvent {
  const DisclosureShareRequestedAttributesApproved();

  @override
  List<Object?> get props => [];
}

class DisclosurePinConfirmed extends DisclosureEvent {
  final DisclosureFlow? flow;

  const DisclosurePinConfirmed(this.flow);

  @override
  List<Object?> get props => [];
}

class DisclosureBackPressed extends DisclosureEvent {
  const DisclosureBackPressed();

  @override
  List<Object?> get props => [];
}

class DisclosureStopRequested extends DisclosureEvent {
  final DisclosureFlow? flow;

  const DisclosureStopRequested({this.flow});

  @override
  List<Object?> get props => [flow];
}

class DisclosureReportPressed extends DisclosureEvent {
  final DisclosureFlow? flow;
  final ReportingOption option;

  const DisclosureReportPressed({this.flow, required this.option});

  @override
  List<Object?> get props => [flow, option];
}
