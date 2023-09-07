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

class CoreRedirectUriError extends CoreError {
  final RedirectError redirectError;

  const CoreRedirectUriError(super.description, {required this.redirectError});

  @override
  List<Object?> get props => [description, redirectError];
}

enum RedirectError { accessDenied, serverError, unknown }
