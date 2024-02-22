/// BLoC States can implement this [NetworkError] class to provide extra info
/// about the error that occurred. To be used by the UI layer
/// to consistently show the most relevant network related [ErrorScreen].
abstract class NetworkError {
  bool get hasInternet;
  int? get statusCode;
}
