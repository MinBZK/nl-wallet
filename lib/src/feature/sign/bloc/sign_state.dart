part of 'sign_bloc.dart';

abstract class SignState extends Equatable {
  bool get showStopConfirmation => true;

  bool get canGoBack => false;

  bool get didGoBack => false;

  double get stepperProgress => 0.0;

  final SignFlow? flow;

  const SignState(this.flow);

  @override
  List<Object?> get props => [showStopConfirmation, canGoBack, didGoBack, stepperProgress, flow];
}

class SignInitial extends SignState {
  const SignInitial() : super(null);
}

class SignLoadInProgress extends SignState {
  const SignLoadInProgress(super.flow);

  @override
  bool get showStopConfirmation => false;
}

class SignCheckOrganization extends SignState {
  final bool afterBackPressed;

  const SignCheckOrganization(SignFlow flow, {this.afterBackPressed = false}) : super(flow);

  @override
  SignFlow get flow => super.flow!;

  @override
  double get stepperProgress => 0.2;

  @override
  bool get didGoBack => afterBackPressed;
}

class SignCheckAgreement extends SignState {
  final bool afterBackPressed;

  const SignCheckAgreement(SignFlow flow, {this.afterBackPressed = false}) : super(flow);

  @override
  SignFlow get flow => super.flow!;

  @override
  double get stepperProgress => 0.4;

  @override
  bool get canGoBack => true;

  @override
  bool get didGoBack => afterBackPressed;
}

class SignConfirmAgreement extends SignState {
  final bool afterBackPressed;

  const SignConfirmAgreement(SignFlow flow, {this.afterBackPressed = false}) : super(flow);

  @override
  SignFlow get flow => super.flow!;

  @override
  double get stepperProgress => 0.6;

  @override
  bool get canGoBack => true;

  @override
  bool get didGoBack => afterBackPressed;
}

class SignConfirmPin extends SignState {
  const SignConfirmPin(SignFlow flow) : super(flow);

  @override
  SignFlow get flow => super.flow!;

  @override
  double get stepperProgress => 0.8;

  @override
  bool get canGoBack => true;
}

class SignSuccess extends SignState {
  const SignSuccess(SignFlow flow) : super(flow);

  @override
  SignFlow get flow => super.flow!;

  @override
  double get stepperProgress => 1;

  @override
  bool get showStopConfirmation => false;
}

class SignError extends SignState {
  const SignError(super.flow);

  @override
  bool get showStopConfirmation => false;
}

class SignStopped extends SignState {
  const SignStopped(super.flow);

  @override
  bool get showStopConfirmation => false;
}
