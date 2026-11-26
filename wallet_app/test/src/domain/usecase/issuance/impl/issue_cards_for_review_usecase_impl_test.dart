import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/issuance/impl/issue_cards_for_review_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/issuance/issue_cards_for_review_usecase.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  final repository = MockIssuanceRepository();
  late IssueCardsForReviewUseCase usecase;

  setUp(() {
    reset(repository);
    usecase = IssueCardsForReviewUseCaseImpl(repository);
  });

  test('invoke returns the issued cards', () async {
    const pin = '123456';
    final cards = <WalletCard>[WalletMockData.card, WalletMockData.altCard];
    when(repository.issueCardsForReview(any)).thenAnswer((_) async => cards);

    final result = await usecase.invoke(pin);

    verify(repository.issueCardsForReview(pin));
    expect(result.hasError, isFalse);
    expect(result.value, cards);
  });

  test('invoke returns error when issuing throws an exception', () async {
    final exception = Exception('Test exception');
    when(repository.issueCardsForReview(any)).thenThrow(exception);

    final result = await usecase.invoke('123456');

    expect(result.hasError, isTrue);
    expect(result.error, isA<GenericError>());
    expect(result.error?.sourceError, equals(exception));
  });

  test('invoke maps an instruction error to a CheckPinError', () async {
    when(repository.issueCardsForReview(any)).thenThrow(const WalletInstructionError.blocked());

    final result = await usecase.invoke('123456');

    expect(result.error, isA<CheckPinError>());
  });
}
