part of 'disclosure_bloc.dart';

abstract class DisclosureEvent extends Equatable {
  const DisclosureEvent();

  @override
  List<Object?> get props => [];
}

class DisclosureOrganizationApproved extends DisclosureEvent {
  const DisclosureOrganizationApproved();
}

class DisclosureShareRequestedAttributesApproved extends DisclosureEvent {
  const DisclosureShareRequestedAttributesApproved();
}

class DisclosurePinConfirmed extends DisclosureEvent {
  const DisclosurePinConfirmed();
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

class DisclosureUpdateState extends DisclosureEvent {
  final DisclosureState state;

  const DisclosureUpdateState(this.state);

  @override
  List<Object?> get props => [state];
}
