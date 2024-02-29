/// BLoC states that represent an error should implement this [ErrorState]
/// class so that all the error information is available and can potentially
/// be used to display the corresponding error UI.
abstract class ErrorState {
  Object get error;
}
