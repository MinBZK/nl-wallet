import 'package:equatable/equatable.dart';

import '../../../wallet_core/error/core_error.dart';
import '../localized_text.dart';
import '../pin/check_pin_result.dart';
import '../pin/pin_validation_error.dart';

sealed class ApplicationError extends Equatable implements Exception {
  final Object sourceError;

  const ApplicationError({required this.sourceError});

  @override
  List<Object?> get props => [sourceError];
}

/// Generic application error
class GenericError extends ApplicationError {
  // Error details, not intended for the end-user.
  final dynamic rawMessage;

  // Error related url, to which we can redirect he user.
  final String? redirectUrl;

  const GenericError(this.rawMessage, {this.redirectUrl, required super.sourceError});

  @override
  List<Object?> get props => [...super.props, rawMessage, redirectUrl];
}

/// Network related error
class NetworkError extends ApplicationError {
  /// Whether the device is connected to the internet
  final bool hasInternet;

  /// StatusCode as returned by the server (if available)
  final int? statusCode;

  const NetworkError({required this.hasInternet, this.statusCode, required super.sourceError});

  @override
  List<Object?> get props => [...super.props, hasInternet, statusCode];
}

/// Pin validation error, relevant when setting up a new pin
class ValidatePinError extends ApplicationError {
  final PinValidationError error;

  const ValidatePinError(this.error, {required super.sourceError});

  @override
  List<Object?> get props => [...super.props, error];
}

/// Check Pin error, provided by the server when the provided pin is not accepted
class CheckPinError extends ApplicationError {
  final CheckPinResult result;

  const CheckPinError(this.result, {required super.sourceError});

  @override
  List<Object?> get props => [...super.props, result];
}

/// Hardware unsupported error, indicates the device does not meet the requirements
class HardwareUnsupportedError extends ApplicationError {
  const HardwareUnsupportedError({required super.sourceError});

  @override
  List<Object?> get props => [...super.props];
}

class SessionError extends ApplicationError {
  final SessionState state;
  final SessionType? crossDevice;
  final bool canRetry;
  final String? returnUrl;

  const SessionError({
    required this.state,
    this.crossDevice,
    this.canRetry = false,
    this.returnUrl,
    required super.sourceError,
  });

  @override
  List<Object?> get props => [...super.props, state, crossDevice, canRetry, returnUrl];
}

enum SessionState { expired, cancelled }

enum SessionType { sameDevice, crossDevice }

class RedirectUriError extends ApplicationError {
  final RedirectError redirectError;

  const RedirectUriError({required this.redirectError, required super.sourceError});

  @override
  List<Object?> get props => [...super.props, redirectError];
}

class ExternalScannerError extends ApplicationError {
  const ExternalScannerError({required super.sourceError});
}

class RelyingPartyError extends ApplicationError {
  final LocalizedText? organizationName;

  const RelyingPartyError({required super.sourceError, this.organizationName});

  @override
  List<Object?> get props => [...super.props, organizationName];
}
