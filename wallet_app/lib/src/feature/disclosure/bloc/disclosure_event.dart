part of 'disclosure_bloc.dart';

abstract class DisclosureEvent extends Equatable {
  const DisclosureEvent();

  @override
  List<Object?> get props => [];
}

class DisclosureSessionStarted extends DisclosureEvent {
  final String uri;
  final bool isQrCode;

  const DisclosureSessionStarted(this.uri, {this.isQrCode = false});

  @override
  List<Object?> get props => [uri, isQrCode];
}

class DisclosureUrlApproved extends DisclosureEvent {
  const DisclosureUrlApproved();
}

class DisclosureShareRequestedCardsApproved extends DisclosureEvent {
  const DisclosureShareRequestedCardsApproved();
}

class DisclosureAlternativeCardSelected extends DisclosureEvent {
  final DiscloseCardRequest updatedCardRequest;

  const DisclosureAlternativeCardSelected(this.updatedCardRequest);

  @override
  List<Object?> get props => [updatedCardRequest];
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

class DisclosureConfirmPinFailed extends DisclosureEvent {
  final ApplicationError error;

  const DisclosureConfirmPinFailed({required this.error});
}
