part of 'recover_pin_bloc.dart';

const kRecoverPinSteps = 4;

sealed class RecoverPinState extends Equatable {
  FlowProgress? get stepperProgress => null;

  bool get canGoBack => false;

  bool get didGoBack => false;

  const RecoverPinState();

  @override
  List<Object?> get props => [stepperProgress, didGoBack, canGoBack];
}

class RecoverPinInitial extends RecoverPinState {
  @override
  final bool didGoBack;

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 1, totalSteps: kRecoverPinSteps);

  const RecoverPinInitial({this.didGoBack = false});
}

class RecoverPinLoadingDigidUrl extends RecoverPinState {
  const RecoverPinLoadingDigidUrl();
}

class RecoverPinAwaitingDigidAuthentication extends RecoverPinState {
  final String authUrl;

  @override
  List<Object?> get props => [...super.props, authUrl];

  const RecoverPinAwaitingDigidAuthentication(this.authUrl);
}

class RecoverPinVerifyingDigidAuthentication extends RecoverPinState {
  const RecoverPinVerifyingDigidAuthentication();
}

class RecoverPinDigidMismatch extends RecoverPinState {
  const RecoverPinDigidMismatch();
}

class RecoverPinStopped extends RecoverPinState {
  const RecoverPinStopped();
}

class RecoverPinChooseNewPin extends RecoverPinState {
  final String authUrl;
  final String pin;
  @override
  final bool didGoBack;

  @override
  bool get canGoBack => true;

  int get enteredDigits => pin.length;

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 2, totalSteps: kRecoverPinSteps);

  const RecoverPinChooseNewPin({required this.authUrl, this.pin = '', this.didGoBack = false})
      : assert(pin.length <= kPinDigits, 'Pin length should never exceed $kPinDigits');

  RecoverPinChooseNewPin copyWith({String? authUrl, String? pin, bool? didGoBack}) {
    return RecoverPinChooseNewPin(
      authUrl: authUrl ?? this.authUrl,
      pin: pin ?? this.pin,
      didGoBack: didGoBack ?? this.didGoBack,
    );
  }

  @override
  List<Object?> get props => [authUrl, pin, ...super.props];
}

class RecoverPinConfirmNewPin extends RecoverPinState {
  final String authUrl;

  /// The user selected pin (during [RecoverPinChooseNewPin])
  final String selectedPin;
  final String pin;

  int get enteredDigits => pin.length;

  // True if this is the second time that the user tries to confirm the [selectedPin]
  final bool isRetrying;

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 3, totalSteps: kRecoverPinSteps);

  @override
  bool get canGoBack => true;

  const RecoverPinConfirmNewPin({
    required this.authUrl,
    this.pin = '',
    required this.selectedPin,
    required this.isRetrying,
  })  : assert(selectedPin.length <= kPinDigits, 'New pin length should never exceed $kPinDigits'),
        assert(pin.length <= kPinDigits, 'Pin length should never exceed $kPinDigits');

  RecoverPinConfirmNewPin copyWith({String? authUrl, String? selectedPin, String? pin, bool? isRetrying}) {
    return RecoverPinConfirmNewPin(
      authUrl: authUrl ?? this.authUrl,
      selectedPin: selectedPin ?? this.selectedPin,
      pin: pin ?? this.pin,
      isRetrying: isRetrying ?? this.isRetrying,
    );
  }

  @override
  List<Object?> get props => [authUrl, pin, selectedPin, isRetrying, ...super.props];
}

class RecoverPinUpdatingPin extends RecoverPinState {
  const RecoverPinUpdatingPin();
}

class RecoverPinSuccess extends RecoverPinState {
  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: kRecoverPinSteps, totalSteps: kRecoverPinSteps);

  const RecoverPinSuccess();
}

class RecoverPinSelectPinFailed extends RecoverPinState implements ErrorState {
  @override
  final ApplicationError error;

  PinValidationError get reason => tryCast<ValidatePinError>(error)?.error ?? PinValidationError.other;

  const RecoverPinSelectPinFailed({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class RecoverPinConfirmPinFailed extends RecoverPinState implements ErrorState {
  @override
  final ApplicationError error;

  final bool canRetry;

  const RecoverPinConfirmPinFailed({required this.error, this.canRetry = true});

  @override
  List<Object?> get props => [error, canRetry, ...super.props];
}

class RecoverPinDigidFailure extends RecoverPinState implements ErrorState {
  @override
  final ApplicationError error;

  const RecoverPinDigidFailure({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class RecoverPinDigidLoginCancelled extends RecoverPinState {
  const RecoverPinDigidLoginCancelled();
}

class RecoverPinNetworkError extends RecoverPinState implements NetworkErrorState {
  @override
  final ApplicationError error;

  @override
  final bool hasInternet;

  @override
  final int? statusCode;

  const RecoverPinNetworkError({required this.error, required this.hasInternet, this.statusCode});

  @override
  List<Object?> get props => [error, hasInternet, statusCode, ...super.props];
}

class RecoverPinGenericError extends RecoverPinState implements ErrorState {
  @override
  final ApplicationError error;

  const RecoverPinGenericError({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class RecoverPinSessionExpired extends RecoverPinState implements ErrorState {
  @override
  final ApplicationError error;

  const RecoverPinSessionExpired({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}
