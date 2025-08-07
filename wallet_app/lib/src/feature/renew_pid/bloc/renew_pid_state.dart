part of 'renew_pid_bloc.dart';

const kRenewPidSteps = 3;

sealed class RenewPidState extends Equatable {
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 0, totalSteps: kRenewPidSteps);

  bool get canGoBack => false;

  bool get didGoBack => false;

  const RenewPidState();

  @override
  List<Object?> get props => [stepperProgress, didGoBack, canGoBack];
}

class RenewPidInitial extends RenewPidState {
  @override
  final bool didGoBack;

  const RenewPidInitial({this.didGoBack = false});

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 0, totalSteps: kRenewPidSteps);
}

class RenewPidLoadingDigidUrl extends RenewPidState {
  const RenewPidLoadingDigidUrl();

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 0, totalSteps: kRenewPidSteps);
}

class RenewPidAwaitingDigidAuthentication extends RenewPidState {
  final String authUrl;

  const RenewPidAwaitingDigidAuthentication(this.authUrl);

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 0, totalSteps: kRenewPidSteps);

  @override
  List<Object?> get props => [...super.props, authUrl];
}

class RenewPidVerifyingDigidAuthentication extends RenewPidState {
  const RenewPidVerifyingDigidAuthentication();

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 0, totalSteps: kRenewPidSteps);
}

class RenewPidDigidMismatch extends RenewPidState {
  const RenewPidDigidMismatch();

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: kRenewPidSteps, totalSteps: kRenewPidSteps);
}

class RenewPidStopped extends RenewPidState {
  const RenewPidStopped();

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: kRenewPidSteps, totalSteps: kRenewPidSteps);
}

class RenewPidCheckData extends RenewPidState {
  final List<Attribute> availableAttributes;
  @override
  final bool didGoBack;

  const RenewPidCheckData({required this.availableAttributes, this.didGoBack = false});

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 1, totalSteps: kRenewPidSteps);

  @override
  List<Object?> get props => [availableAttributes, ...super.props];
}

class RenewPidConfirmPin extends RenewPidState {
  /// Used to return to [RenewPidCheckData] when user presses back
  final List<Attribute> attributes;

  const RenewPidConfirmPin(this.attributes);

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 2, totalSteps: kRenewPidSteps);

  @override
  bool get canGoBack => true;

  @override
  List<Object?> get props => [attributes, ...super.props];
}

class RenewPidUpdatingCards extends RenewPidState {
  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 2, totalSteps: kRenewPidSteps);

  const RenewPidUpdatingCards();
}

class RenewPidSuccess extends RenewPidState {
  final List<WalletCard> addedCards;

  const RenewPidSuccess(this.addedCards);

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: kRenewPidSteps, totalSteps: kRenewPidSteps);

  @override
  List<Object?> get props => [addedCards, ...super.props];
}

class RenewPidDigidFailure extends RenewPidState implements ErrorState {
  @override
  final ApplicationError error;

  const RenewPidDigidFailure({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class RenewPidDigidLoginCancelled extends RenewPidState {
  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 3, totalSteps: kRenewPidSteps);

  const RenewPidDigidLoginCancelled();
}

class RenewPidNetworkError extends RenewPidState implements NetworkErrorState {
  @override
  final ApplicationError error;

  @override
  final bool hasInternet;

  @override
  final int? statusCode;

  const RenewPidNetworkError({required this.error, required this.hasInternet, this.statusCode});

  @override
  List<Object?> get props => [error, hasInternet, statusCode, ...super.props];
}

class RenewPidGenericError extends RenewPidState implements ErrorState {
  @override
  final ApplicationError error;

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: kRenewPidSteps, totalSteps: kRenewPidSteps);

  const RenewPidGenericError({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class RenewPidSessionExpired extends RenewPidState implements ErrorState {
  @override
  final ApplicationError error;

  const RenewPidSessionExpired({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}
