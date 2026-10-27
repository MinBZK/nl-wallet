import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/issuance/continue_issuance_result.dart';
import '../../../domain/model/issuance/start_issuance_result.dart';

export '../../../domain/model/disclosure/start_disclosure_result.dart';

abstract class IssuanceRepository {
  Future<StartIssuanceResult> startIssuance(String disclosureUri, {required bool isQrCode});

  Future<StartIssuanceResult> startIssuanceFromOffer(String disclosureUri, {required bool isQrCode});

  Future<List<WalletCard>> discloseForIssuance(String pin, List<int> selectedIndices);

  Future<ContinueIssuanceResult> continueIssuance(String uri);

  Future<void> acceptIssuance(String pin, Iterable<WalletCard> cards);

  /// Requests the cards from an issuer that does not provide previews, returns them for review.
  /// The cards are not added to the wallet until [acceptIssuedCards] is called.
  Future<List<WalletCard>> issueCardsForReview(String pin);

  /// Adds the cards returned by [issueCardsForReview] to the wallet.
  Future<void> acceptIssuedCards();
}
