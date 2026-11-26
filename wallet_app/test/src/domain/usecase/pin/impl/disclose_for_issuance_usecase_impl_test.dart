import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/issuance/disclose_for_issuance_result.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/pin/disclose_for_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/impl/disclose_for_issuance_usecase_impl.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  const pin = '123456';
  const selectedIndices = [1, 0, 2];

  late MockIssuanceRepository repository;
  late DiscloseForIssuanceUseCase usecase;

  setUp(() {
    repository = MockIssuanceRepository();
    usecase = DiscloseForIssuanceUseCaseImpl(repository, selectedIndices);
  });

  test('invoke forwards the pin and selected indices to the repository', () async {
    when(repository.discloseForIssuance(any, any))
        .thenAnswer((_) async => const DiscloseForIssuanceResult.previews([]));

    await usecase.invoke(pin);

    verify(repository.discloseForIssuance(pin, selectedIndices)).called(1);
  });

  test('invoke returns the previews offered by the repository', () async {
    final cards = <WalletCard>[WalletMockData.card, WalletMockData.altCard];
    final previews = DiscloseForIssuanceResult.previews(cards);
    when(repository.discloseForIssuance(any, any)).thenAnswer((_) async => previews);

    final result = await usecase.invoke(pin);

    expect(result.hasError, isFalse);
    expect(result.value, previews);
  });

  test('invoke returns the cards issued for review by the repository', () async {
    final cards = <WalletCard>[WalletMockData.card, WalletMockData.altCard];
    final issued = DiscloseForIssuanceResult.issuedForReview(cards);
    when(repository.discloseForIssuance(any, any)).thenAnswer((_) async => issued);

    final result = await usecase.invoke(pin);

    expect(result.hasError, isFalse);
    expect(result.value, issued);
  });

  test('invoke returns a GenericError when the repository throws an unknown exception', () async {
    final exception = Exception('Test exception');
    when(repository.discloseForIssuance(any, any)).thenThrow(exception);

    final result = await usecase.invoke(pin);

    expect(result.hasError, isTrue);
    expect(result.error, isA<GenericError>());
    expect(result.error?.sourceError, exception);
  });

  test('invoke maps a CoreError to the matching ApplicationError', () async {
    const coreError = CoreExpiredSessionError('test error', canRetry: false);
    when(repository.discloseForIssuance(any, any)).thenThrow(coreError);

    final result = await usecase.invoke(pin);

    expect(result.hasError, isTrue);
    expect(result.error, isA<SessionError>());
    expect(result.error?.sourceError, coreError);
  });

  test('invoke passes through an ApplicationError thrown by the repository', () async {
    const error = NetworkError(hasInternet: false, sourceError: 'offline');
    when(repository.discloseForIssuance(any, any)).thenThrow(error);

    final result = await usecase.invoke(pin);

    expect(result.error, error);
  });
}
