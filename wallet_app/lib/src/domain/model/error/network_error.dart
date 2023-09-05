/// BLoC States can implement this [NetworkError] class to provide extra info
/// about the error dat occurred. To be used by the UI layer
/// to consistently show the most relevant [ErrorScreen].
abstract class NetworkError {
  bool get hasInternet;
  int? get statusCode;
}
