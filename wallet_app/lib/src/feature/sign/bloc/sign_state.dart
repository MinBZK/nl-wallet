part of 'sign_bloc.dart';

sealed class SignState extends Equatable {
  bool get showStopConfirmation => true;

  bool get canGoBack => false;

  bool get didGoBack => false;

  double get stepperProgress => 0.0;

  const SignState();

  @override
  List<Object?> get props => [
        showStopConfirmation,
        canGoBack,
        didGoBack,
        stepperProgress,
      ];
}

class SignInitial extends SignState {
  const SignInitial();
}

class SignLoadInProgress extends SignState {
  const SignLoadInProgress();

  @override
  bool get showStopConfirmation => false;
}

class SignCheckOrganization extends SignState {
  final Organization organization;

  final bool afterBackPressed;

  const SignCheckOrganization({
    required this.organization,
    this.afterBackPressed = false,
  });

  @override
  double get stepperProgress => 0.2;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  List<Object?> get props => [organization, ...super.props];
}

class SignCheckAgreement extends SignState {
  final bool afterBackPressed;

  final Organization organization;
  final Organization trustProvider;
  final Document document;

  const SignCheckAgreement({
    required this.organization,
    required this.trustProvider,
    required this.document,
    this.afterBackPressed = false,
  });

  @override
  double get stepperProgress => 0.4;

  @override
  bool get canGoBack => true;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  List<Object?> get props => [organization, trustProvider, document, ...super.props];
}

class SignConfirmAgreement extends SignState {
  final bool afterBackPressed;

  final Policy policy;
  final Organization trustProvider;
  final Document document;
  final List<Attribute> requestedAttributes;

  const SignConfirmAgreement({
    required this.policy,
    required this.trustProvider,
    required this.document,
    required this.requestedAttributes,
    this.afterBackPressed = false,
  });

  @override
  double get stepperProgress => 0.6;

  @override
  bool get canGoBack => true;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  List<Object?> get props => [policy, trustProvider, document, requestedAttributes, ...super.props];
}

class SignConfirmPin extends SignState {
  const SignConfirmPin();

  @override
  double get stepperProgress => 0.8;

  @override
  bool get canGoBack => true;
}

class SignSuccess extends SignState {
  final Organization organization;

  const SignSuccess({required this.organization});

  @override
  double get stepperProgress => 1;

  @override
  bool get showStopConfirmation => false;

  @override
  List<Object?> get props => [organization, ...super.props];
}

class SignError extends SignState {
  const SignError();

  @override
  bool get showStopConfirmation => false;
}

class SignStopped extends SignState {
  const SignStopped();

  @override
  bool get showStopConfirmation => false;
}
