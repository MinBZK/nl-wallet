import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';

void main() {
  setUp(() {});

  test('value is available through .value method', () async {
    final result = Result.success(true);
    expect(result.value, isTrue);
  });

  test('toString prints the contained value', () async {
    final result = Result.success('[test message]');
    expect(result.toString(), contains('[test message]'));
  });

  test('toString prints the contained error', () async {
    final error = GenericError('[rawMessage]', sourceError: Exception('[exception]'));
    final result = Result.error(error);
    expect(result.toString(), contains('[rawMessage]'));
    expect(result.toString(), contains('[exception]'));
  });

  test('error is available through .error method', () async {
    final error = GenericError('rawMessage', sourceError: Exception('exception'));
    final result = Result.error(error);
    expect(result.error, error);
  });

  test('error is null when trying to fetch an error from a success result', () async {
    final result = Result.success('success');
    expect(result.error, isNull);
  });

  test('hasError behaves as expected', () async {
    final error = GenericError('rawMessage', sourceError: Exception('exception'));
    final errorResult = Result.error(error);
    final successResult = Result.success(true);

    expect(errorResult.hasError, isTrue);
    expect(successResult.hasError, isFalse);
  });
}
