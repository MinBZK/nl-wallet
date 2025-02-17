import 'package:equatable/equatable.dart';

sealed class CoreError extends Equatable {
  final String? description;
  final Map<String, dynamic>? data;

  const CoreError(this.description, {this.data});

  @override
  List<Object?> get props => [description, data];
}

class CoreGenericError extends CoreError {
  const CoreGenericError(super.description, {super.data});
}

class CoreNetworkError extends CoreError {
  const CoreNetworkError(super.description, {super.data});
}

class CoreStateError extends CoreError {
  const CoreStateError(super.description, {super.data});
}

class CoreRedirectUriError extends CoreError {
  final RedirectError redirectError;

  const CoreRedirectUriError(super.description, {super.data, required this.redirectError});

  @override
  List<Object?> get props => [redirectError, ...super.props];
}

enum RedirectError { accessDenied, serverError, loginRequired, unknown }

class CoreHardwareKeyUnsupportedError extends CoreError {
  const CoreHardwareKeyUnsupportedError(super.description, {super.data});
}

class CoreDisclosureSourceMismatchError extends CoreError {
  final bool isCrossDevice;

  const CoreDisclosureSourceMismatchError(super.description, {super.data, required this.isCrossDevice});

  @override
  List<Object?> get props => [isCrossDevice, ...super.props];
}

class CoreExpiredSessionError extends CoreError {
  final bool canRetry;

  const CoreExpiredSessionError(super.description, {super.data, required this.canRetry});

  @override
  List<Object?> get props => [canRetry, ...super.props];
}

class CoreCancelledSessionError extends CoreError {
  const CoreCancelledSessionError(super.description, {super.data});
}
