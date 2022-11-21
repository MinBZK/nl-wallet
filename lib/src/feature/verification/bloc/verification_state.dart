part of 'verification_bloc.dart';

abstract class VerificationState extends Equatable {
  bool get showStopConfirmation => true;

  bool get canGoBack => false;

  bool get didGoBack => false;

  double get stepperProgress => 0.0;

  Organization? get organization => null;

  const VerificationState();

  @override
  List<Object?> get props => [showStopConfirmation, canGoBack, didGoBack, stepperProgress, organization];
}

class VerificationInitial extends VerificationState {}

class VerificationLoadInProgress extends VerificationState {}

class VerificationGenericError extends VerificationState {
  @override
  bool get showStopConfirmation => false;
}

class VerificationCheckOrganization extends VerificationState {
  final VerificationRequest request;
  final bool afterBackPressed;

  const VerificationCheckOrganization(this.request, {this.afterBackPressed = false});

  @override
  List<Object?> get props => [request, ...super.props];

  @override
  double get stepperProgress => 0.25;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  Organization? get organization => request.organization;
}

class VerificationMissingAttributes extends VerificationState {
  final VerificationRequest request;

  const VerificationMissingAttributes(this.request);

  @override
  List<Object?> get props => [request, ...super.props];

  @override
  double get stepperProgress => 0.5;

  @override
  bool get canGoBack => true;

  @override
  bool get showStopConfirmation => false;

  @override
  Organization? get organization => request.organization;
}

class VerificationConfirmDataAttributes extends VerificationState {
  final VerificationRequest request;
  final bool afterBackPressed;

  const VerificationConfirmDataAttributes(this.request, {this.afterBackPressed = false});

  @override
  List<Object?> get props => [request, ...super.props];

  @override
  double get stepperProgress => 0.5;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  Organization? get organization => request.organization;

  @override
  bool get canGoBack => true;
}

class VerificationConfirmPin extends VerificationState {
  final VerificationRequest request;

  const VerificationConfirmPin(this.request);

  @override
  List<Object?> get props => [request, ...super.props];

  @override
  double get stepperProgress => 0.75;

  @override
  Organization? get organization => request.organization;

  @override
  bool get canGoBack => true;
}

class VerificationSuccess extends VerificationState {
  final VerificationRequest request;

  const VerificationSuccess(this.request);

  @override
  List<Object?> get props => [request, ...super.props];

  @override
  double get stepperProgress => 1;

  @override
  Organization? get organization => request.organization;

  @override
  bool get showStopConfirmation => false;
}

class VerificationStopped extends VerificationState {
  const VerificationStopped();

  @override
  double get stepperProgress => 1;

  @override
  bool get showStopConfirmation => false;
}
