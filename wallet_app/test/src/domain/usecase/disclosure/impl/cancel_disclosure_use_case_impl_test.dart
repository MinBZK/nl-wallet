import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/disclosure/cancel_disclosure_usecase.dart';
import 'package:wallet/src/domain/usecase/disclosure/impl/cancel_disclosure_usecase_impl.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  final repository = MockDisclosureRepository();
  late CancelDisclosureUseCase usecase;

  setUp(() {
    usecase = CancelDisclosureUseCaseImpl(repository);
  });

  test('cancel is called when session is active', () async {
    when(repository.hasActiveDisclosureSession()).thenAnswer((_) async => true);
    await usecase.invoke();
    verify(repository.cancelDisclosure()).called(1);
  });

  test('cancel is not called when session is not active', () async {
    when(repository.hasActiveDisclosureSession()).thenAnswer((_) async => false);
    await usecase.invoke();
    verifyNever(repository.cancelDisclosure());
  });

  test('when cancelDisclosure throws CoreError with a returnUrl, invoke returns success with that url', () async {
    when(repository.hasActiveDisclosureSession()).thenAnswer((_) async => true);

    const mockReturnUrl = 'https://example.org';
    final coreErrorWithReturnUrl = const CoreGenericError('test', data: {'return_url': mockReturnUrl});
    when(repository.cancelDisclosure()).thenThrow(coreErrorWithReturnUrl);

    final result = await usecase.invoke();

    expect(result.value, mockReturnUrl, reason: 'The success value should be the returnUrl from the CoreError.');
    expect(
      result.hasError,
      isFalse,
      reason: 'Result should not be an error despite the underlying CoreError, due to the presence of returnUrl.',
    );
  });

  test('when cancelDisclosure throws CoreError without a returnUrl, invoke returns error', () async {
    when(repository.hasActiveDisclosureSession()).thenAnswer((_) async => true);

    final coreErrorWithoutReturnUrl = const CoreGenericError('test');
    when(repository.cancelDisclosure()).thenThrow(coreErrorWithoutReturnUrl);

    final result = await usecase.invoke();

    expect(
      result.hasError,
      isTrue,
      reason: 'Result should be an error because CoreError was thrown without a returnUrl.',
    );
    expect(
      result.error,
      isA<GenericError>(),
      reason: 'The error should be a GenericError, as mapped from CoreGenericError by asApplicationError().',
    );
  });

  test('when cancelDisclosure throws any other error, invoke returns error', () async {
    when(repository.hasActiveDisclosureSession()).thenAnswer((_) async => true);

    when(repository.cancelDisclosure()).thenThrow(Exception('test exception'));

    final result = await usecase.invoke();

    expect(
      result.hasError,
      isTrue,
      reason: 'Result should be an error because CoreError was thrown without a returnUrl.',
    );
    expect(
      result.error,
      isA<GenericError>(),
      reason: 'The error should be a GenericError, as mapped from CoreGenericError by asApplicationError().',
    );
  });
}
