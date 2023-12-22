part of 'disclosure_bloc.dart';

abstract class DisclosureEvent extends Equatable {
  const DisclosureEvent();

  @override
  List<Object?> get props => [];
}

class DisclosureSessionStarted extends DisclosureEvent {
  final String uri;

  const DisclosureSessionStarted(this.uri);

  @override
  List<Object?> get props => [uri];
}

class DisclosureOrganizationApproved extends DisclosureEvent {
  const DisclosureOrganizationApproved();
}

class DisclosureShareRequestedAttributesApproved extends DisclosureEvent {
  const DisclosureShareRequestedAttributesApproved();
}

class DisclosurePinConfirmed extends DisclosureEvent {
  final String? returnUrl;

  const DisclosurePinConfirmed({this.returnUrl});

  @override
  List<Object?> get props => [returnUrl];
}

class DisclosureBackPressed extends DisclosureEvent {
  const DisclosureBackPressed();
}

class DisclosureStopRequested extends DisclosureEvent {
  const DisclosureStopRequested();
}

class DisclosureReportPressed extends DisclosureEvent {
  final ReportingOption option;

  const DisclosureReportPressed({required this.option});

  @override
  List<Object?> get props => [option];
}
