import '../../model/card/wallet_card.dart';
import '../pin/check_pin_usecase.dart';

/// Requests the cards from an issuer that does not provide previews, returns the issued cards for review.
/// The cards are not added to the wallet until the user accepts them, see [CommitIssuedCardsUseCase].
abstract class IssueCardsForReviewUseCase extends CheckPinUseCase {
  @override
  Future<Result<List<WalletCard>>> invoke(String pin);
}
