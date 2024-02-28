import 'error_state.dart';

/// BLoC States that represent a network error should implement this [NetworkErrorState] class
/// to provide extra info about the error that occurred. To be used by the UI layer
/// to consistently show the most relevant network related [ErrorScreen].
abstract class NetworkErrorState extends ErrorState {
  bool get hasInternet;

  int? get statusCode;
}
