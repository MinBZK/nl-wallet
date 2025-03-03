import 'package:equatable/equatable.dart';

import 'card/wallet_card.dart';
import 'issuance_response.dart';
import 'organization.dart';

class MultipleCardsFlow extends Equatable {
  final int activeIndex;
  final Map<WalletCard, Organization> cardToOrganizations;
  final Set<String> selectedCardIds;

  List<WalletCard> get availableCards => cardToOrganizations.keys.toList();

  List<WalletCard> get selectedCards => availableCards.where((card) => selectedCardIds.contains(card.id)).toList();

  WalletCard get activeCard => selectedCards[activeIndex];

  bool get hasMoreCards => activeIndex < (selectedCardIds.length - 1);

  bool get isAtFirstCard => activeIndex == 0;

  const MultipleCardsFlow({
    required this.cardToOrganizations,
    required this.selectedCardIds,
    required this.activeIndex,
  });

  factory MultipleCardsFlow.fromIssuance(List<IssuanceResponse> responses) {
    final cardToOrganizations = <WalletCard, Organization>{};
    for (final response in responses) {
      for (final card in response.cards) {
        cardToOrganizations[card] = response.organization;
      }
    }
    return MultipleCardsFlow(
      cardToOrganizations: cardToOrganizations,
      selectedCardIds: cardToOrganizations.keys.map((e) => e.id).toSet(),
      activeIndex: 0,
    );
  }

  factory MultipleCardsFlow.fromCards(List<WalletCard> cards, Organization organization) {
    final cardToOrganizations = <WalletCard, Organization>{};
    for (final card in cards) {
      cardToOrganizations[card] = organization;
    }
    return MultipleCardsFlow(
      cardToOrganizations: cardToOrganizations,
      selectedCardIds: cardToOrganizations.keys.map((e) => e.id).toSet(),
      activeIndex: 0,
    );
  }

  MultipleCardsFlow previous() {
    if (activeIndex <= 0) throw UnsupportedError('Already at first element');
    return copyWith(activeIndex: activeIndex - 1);
  }

  MultipleCardsFlow next() {
    if (activeIndex >= (selectedCardIds.length - 1)) throw UnsupportedError('Already at last element');
    return copyWith(activeIndex: activeIndex + 1);
  }

  @override
  List<Object?> get props => [activeIndex, cardToOrganizations, selectedCardIds];

  MultipleCardsFlow copyWith({
    int? activeIndex,
    Map<WalletCard, Organization>? cardToOrganizations,
    Set<String>? selectedCardIds,
  }) {
    return MultipleCardsFlow(
      activeIndex: activeIndex ?? this.activeIndex,
      cardToOrganizations: cardToOrganizations ?? this.cardToOrganizations,
      selectedCardIds: selectedCardIds ?? this.selectedCardIds,
    );
  }
}
