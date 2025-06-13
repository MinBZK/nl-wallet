import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/issuance/bloc/issuance_bloc.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  final newCard = WalletMockData.card.copyWith(id: () => null);
  final updatedCard = WalletMockData.altCard;

  group('IssuanceReviewCards', () {
    test('init factory selects all cards', () {
      final cards = [newCard, updatedCard];
      final state = IssuanceReviewCards.init(cards: cards);

      expect(state.selectableCards.length, 2);
      expect(cards.every((card) => state.selectableCards[card]!), isTrue);
      expect(state.selectedCards, cards);
    });

    test('toggleCard flips the selection state of a card', () {
      final cards = [newCard];
      final initial = IssuanceReviewCards.init(cards: cards);

      final toggled = initial.toggleCard(newCard);
      expect(toggled.selectableCards[newCard], false);

      final toggledAgain = toggled.toggleCard(newCard);
      expect(toggledAgain.selectableCards[newCard], true);
    });

    test('offeredCards returns newly created cards', () {
      final state = IssuanceReviewCards(
        selectableCards: {
          newCard: true,
          updatedCard: true,
        },
      );

      expect(state.offeredCards, contains(newCard));
      expect(state.offeredCards, isNot(contains(updatedCard)));
    });

    test('renewedCards returns updated cards', () {
      final state = IssuanceReviewCards(
        selectableCards: {
          newCard: true,
          updatedCard: true,
        },
      );

      expect(state.renewedCards, isNot(contains(newCard)));
      expect(state.renewedCards, contains(updatedCard));
    });

    test('selectedCards filters correctly', () {
      final newCard2 = newCard.copyWith(attributes: [], metadata: []);
      final state = IssuanceReviewCards(
        selectableCards: {
          newCard: true,
          updatedCard: true,
          newCard2: false,
        },
      );

      expect(state.selectedCards, hasLength(2));
      expect(state.selectedCards, contains(newCard));
      expect(state.selectedCards, contains(updatedCard));
      expect(state.selectedCards, isNot(contains(newCard2)));
    });
  });
}
