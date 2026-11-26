import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/issuance/commit_issued_cards_usecase.dart';
import 'package:wallet/src/domain/usecase/issuance/impl/commit_issued_cards_usecase_impl.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  final repository = MockIssuanceRepository();
  late CommitIssuedCardsUseCase usecase;

  setUp(() {
    reset(repository);
    usecase = CommitIssuedCardsUseCaseImpl(repository);
  });

  test('invoke calls commitIssuedCards and returns success', () async {
    when(repository.commitIssuedCards()).thenAnswer((_) async {});

    final result = await usecase.invoke();

    verify(repository.commitIssuedCards());
    expect(result.hasError, isFalse);
  });

  test('invoke returns error when commitIssuedCards throws an exception', () async {
    final exception = Exception('Test exception');
    when(repository.commitIssuedCards()).thenThrow(exception);

    final result = await usecase.invoke();

    expect(result.hasError, isTrue);
    expect(result.error, isA<GenericError>());
    expect(result.error?.sourceError, equals(exception));
  });

  test('invoke handles CoreError by mapping to appropriate ApplicationError', () async {
    when(repository.commitIssuedCards()).thenThrow(const CoreExpiredSessionError('test error', canRetry: false));

    final result = await usecase.invoke();

    expect(result.error, isA<SessionError>());
  });
}
