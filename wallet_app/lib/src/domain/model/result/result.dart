import '../../../util/cast_util.dart';
import 'application_error.dart';

sealed class Result<T> {
  const Result();

  const factory Result.success(T value) = Success._;

  const factory Result.error(ApplicationError error) = Error._;

  Future<void> process({required Function(T) onSuccess, required Function(ApplicationError) onError}) async {
    switch (this) {
      case Success<T>():
        await onSuccess((this as Success<T>).value);
      case Error<T>():
        await onError((this as Error<T>).error);
    }
  }

  T? get value => tryCast<Success<T>>(this)?.value;

  ApplicationError? get error => tryCast<Error<T>>(this)?.error;

  bool get hasError => this is Error<T>;
}

final class Success<T> extends Result<T> {
  const Success._(this.value);

  @override
  final T value;

  @override
  String toString() => 'Result<$T>.success($value)';
}

final class Error<T> extends Result<T> {
  const Error._(this.error);

  @override
  final ApplicationError error;

  @override
  String toString() => 'Result<$T>.error($error)';
}
