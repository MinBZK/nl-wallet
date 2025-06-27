import 'package:equatable/equatable.dart';

import '../card/wallet_card.dart';

/// Represents a request for the user to share a card, where one or more candidate cards are available.
/// The user may select the specific card to share from the provided candidates.
class DiscloseCardRequest extends Equatable {
  /// The index of the currently selected card in the [candidates] list.
  final int selectedIndex;

  /// The list of candidate wallet cards available for selection.
  /// Must not be empty, as required by the constructor.
  final List<WalletCard> candidates;

  /// Returns the currently selected card
  WalletCard get selection => candidates[selectedIndex];

  /// Returns a list of candidate cards excluding the currently selected card.
  List<WalletCard> get alternatives => [...candidates]..remove(selection);

  /// Returns whether there are alternative cards available for selection.
  bool get hasAlternatives => candidates.length > 1;

  /// Unique identifier for the card request, derived from the hash code of the [candidates] list.
  /// This ensures that different candidate configurations result in distinct IDs.
  int get id => candidates.hashCode;

  DiscloseCardRequest({required this.candidates, this.selectedIndex = 0})
      : assert(
          selectedIndex >= 0 && selectedIndex < candidates.length,
          'selectedIndex must be within valid range of candidates list',
        ),
        assert(candidates.isNotEmpty, 'At least one candidate should be provided');

  /// Creates a DisclosureCardRequest for a single card with no alternatives
  factory DiscloseCardRequest.fromCard(WalletCard card) => DiscloseCardRequest(candidates: [card]);

  /// Creates a new CardRequest with the specified card selected
  /// Returns the same instance if the card is not found in candidates
  DiscloseCardRequest select(WalletCard card) {
    final index = candidates.indexOf(card);
    return index >= 0 ? DiscloseCardRequest(candidates: candidates, selectedIndex: index) : this;
  }

  @override
  List<Object?> get props => [selectedIndex, candidates];
}
