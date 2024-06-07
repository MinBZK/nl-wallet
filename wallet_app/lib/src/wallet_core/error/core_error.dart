import 'package:equatable/equatable.dart';

sealed class CoreError extends Equatable {
  final String? description;

  const CoreError(this.description);

  @override
  List<Object?> get props => [description];
}

class CoreGenericError extends CoreError {
  const CoreGenericError(super.description);
}

class CoreNetworkError extends CoreError {
  const CoreNetworkError(super.description);
}

class CoreStateError extends CoreError {
  final Map<String, dynamic>? data;

  const CoreStateError(super.description, this.data);

  @override
  List<Object?> get props => [data, ...super.props];
}

class CoreRedirectUriError extends CoreError {
  final RedirectError redirectError;

  const CoreRedirectUriError(super.description, {required this.redirectError});

  @override
  List<Object?> get props => [redirectError, ...super.props];
}

enum RedirectError { accessDenied, serverError, unknown }

class CoreHardwareKeyUnsupportedError extends CoreError {
  const CoreHardwareKeyUnsupportedError(super.description);
}

class CoreDisclosureSourceMismatchError extends CoreError {
  final bool isCrossDevice;

  const CoreDisclosureSourceMismatchError(super.description, {required this.isCrossDevice});

  @override
  List<Object?> get props => [isCrossDevice, ...super.props];
}
