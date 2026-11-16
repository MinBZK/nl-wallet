import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/issuance/core/core_issuance_repository.dart';
import 'package:wallet/src/domain/model/issuance/disclose_for_issuance_result.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late CoreIssuanceRepository repository;
  late MockTypedWalletCore mockTypedWalletCore;

  setUp(() {
    mockTypedWalletCore = MockTypedWalletCore();
    repository = CoreIssuanceRepository(
      mockTypedWalletCore,
      MockMapper(),
      MockMapper(),
      MockMapper(),
      MockMapper(),
      MockMapper(),
    );
  });

  group('issueCardsForReview', () {
    test('pin is forwarded to wallet core and the issued cards are returned', () async {
      when(mockTypedWalletCore.issueCredentialsForReview(any)).thenAnswer(
        (_) async => const IssueCredentialsResult.ok([]),
      );

      final cards = await repository.issueCardsForReview('123123');

      verify(mockTypedWalletCore.issueCredentialsForReview('123123')).called(1);
      expect(cards, isEmpty);
    });

    test('instruction error is thrown', () async {
      const error = WalletInstructionError.blocked();
      when(mockTypedWalletCore.issueCredentialsForReview(any)).thenAnswer(
        (_) async => const IssueCredentialsResult.instructionError(error: error),
      );

      expect(() => repository.issueCardsForReview('123123'), throwsA(error));
    });
  });

  group('discloseForIssuance', () {
    test('previews are returned as previews', () async {
      when(mockTypedWalletCore.continueDisclosureBasedIssuance(any, any)).thenAnswer(
        (_) async => const DisclosureBasedIssuanceInstructionResult.ok(DisclosureBasedIssuanceResult.previews([])),
      );

      final result = await repository.discloseForIssuance('123123', [0]);

      expect(result, const DiscloseForIssuanceResult.previews([]));
    });

    test('no issuable attestations results in empty previews', () async {
      when(mockTypedWalletCore.continueDisclosureBasedIssuance(any, any)).thenAnswer(
        (_) async => const DisclosureBasedIssuanceInstructionResult.ok(
          DisclosureBasedIssuanceResult.noIssuableAttestations(),
        ),
      );

      final result = await repository.discloseForIssuance('123123', [0]);

      expect(result, const DiscloseForIssuanceResult.previews([]));
    });

    test('cards issued without previews are returned for review', () async {
      when(mockTypedWalletCore.continueDisclosureBasedIssuance(any, any)).thenAnswer(
        (_) async => const DisclosureBasedIssuanceInstructionResult.ok(
          DisclosureBasedIssuanceResult.issuedForReview([]),
        ),
      );

      final result = await repository.discloseForIssuance('123123', [0]);

      expect(result, const DiscloseForIssuanceResult.issuedForReview([]));
    });
  });

  group('commitIssuedCards', () {
    test('call is forwarded to wallet core', () async {
      await repository.commitIssuedCards();

      verify(mockTypedWalletCore.commitIssuedCredentials()).called(1);
    });
  });
}
