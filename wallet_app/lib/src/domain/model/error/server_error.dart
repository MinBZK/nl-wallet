/// BLoC States can implement this [ServerError] class to provide extra info
/// about the error dat occurred. To be used by the UI layer
/// to consistently show the most relevant [ErrorScreen].
abstract class ServerError {
  int? get statusCode;
}
