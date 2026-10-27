import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/issuance/accept_issued_cards_usecase.dart';
import 'package:wallet/src/domain/usecase/issuance/impl/accept_issued_cards_usecase_impl.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  final repository = MockIssuanceRepository();
  late AcceptIssuedCardsUseCase usecase;

  setUp(() {
    reset(repository);
    usecase = AcceptIssuedCardsUseCaseImpl(repository);
  });

  test('invoke calls acceptIssuedCards and returns success', () async {
    when(repository.acceptIssuedCards()).thenAnswer((_) async {});

    final result = await usecase.invoke();

    verify(repository.acceptIssuedCards());
    expect(result.hasError, isFalse);
  });

  test('invoke returns error when acceptIssuedCards throws an exception', () async {
    final exception = Exception('Test exception');
    when(repository.acceptIssuedCards()).thenThrow(exception);

    final result = await usecase.invoke();

    expect(result.hasError, isTrue);
    expect(result.error, isA<GenericError>());
    expect(result.error?.sourceError, equals(exception));
  });

  test('invoke handles CoreError by mapping to appropriate ApplicationError', () async {
    when(repository.acceptIssuedCards()).thenThrow(const CoreExpiredSessionError('test error', canRetry: false));

    final result = await usecase.invoke();

    expect(result.error, isA<SessionError>());
  });
}
