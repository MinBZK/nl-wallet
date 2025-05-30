import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/issuance/accept_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/issuance/impl/accept_issuance_usecase_impl.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  final repository = MockIssuanceRepository();
  final mockCards = <WalletCard>[WalletMockData.card, WalletMockData.altCard];
  late AcceptIssuanceUseCase usecase;

  setUp(() {
    reset(repository);
    usecase = AcceptIssuanceUseCaseImpl(repository, cards: mockCards);
  });

  test('invoke calls acceptIssuance with correct parameters', () async {
    const pin = '123456';
    when(repository.acceptIssuance(any, any)).thenAnswer((_) async {});

    await usecase.invoke(pin);

    verify(repository.acceptIssuance(pin, mockCards));
  });

  test('invoke returns success when acceptIssuance completes without errors', () async {
    const pin = '123456';
    when(repository.acceptIssuance(any, any)).thenAnswer((_) async {});

    final result = await usecase.invoke(pin);

    expect(result.hasError, isFalse);
  });

  test('invoke returns error when acceptIssuance throws an exception', () async {
    const pin = '123456';
    final exception = Exception('Test exception');
    when(repository.acceptIssuance(any, any)).thenThrow(exception);

    final result = await usecase.invoke(pin);

    expect(result.hasError, isTrue);
    expect(result.error, isA<GenericError>());
    expect(result.error?.sourceError, equals(exception));
  });

  test('invoke handles CoreError by mapping to appropriate ApplicationError', () async {
    const pin = '123456';
    final coreError = const CoreExpiredSessionError('test error', canRetry: false);
    when(repository.acceptIssuance(any, any)).thenThrow(coreError);

    final result = await usecase.invoke(pin);

    expect(result.hasError, isTrue);
    expect(result.error, isA<SessionError>());
  });
}
