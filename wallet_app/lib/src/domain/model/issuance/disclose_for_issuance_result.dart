import 'package:freezed_annotation/freezed_annotation.dart';

import '../card/wallet_card.dart';

part 'disclose_for_issuance_result.freezed.dart';

/// The outcome of disclosing attributes for issuance.
@freezed
sealed class DiscloseForIssuanceResult with _$DiscloseForIssuanceResult {
  /// Previews of the cards on offer. An empty list means the issuer has no cards to offer.
  const factory DiscloseForIssuanceResult.previews(
    List<WalletCard> cards,
  ) = DiscloseForIssuancePreviews;

  /// The issuer does not provide previews, so the cards were issued right after disclosing.
  /// They are not added to the wallet until the user accepts them.
  const factory DiscloseForIssuanceResult.issuedForReview(
    List<WalletCard> cards,
  ) = DiscloseForIssuanceIssuedForReview;
}
