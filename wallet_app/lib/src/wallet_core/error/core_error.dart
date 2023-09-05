sealed class CoreError {
  final String? description;

  CoreError(this.description);
}

class CoreGenericError extends CoreError {
  CoreGenericError(super.description);
}

class CoreNetworkError extends CoreError {
  CoreNetworkError(super.description);
}

class CoreRedirectUriError extends CoreError {
  final RedirectError redirectError;

  CoreRedirectUriError(super.description, {required this.redirectError});
}

enum RedirectError { accessDenied, serverError, unknown }
