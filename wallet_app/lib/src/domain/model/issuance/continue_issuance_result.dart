import 'package:freezed_annotation/freezed_annotation.dart';

import '../card/wallet_card.dart';
import '../organization.dart';

part 'continue_issuance_result.freezed.dart';

@freezed
sealed class ContinueIssuanceResult with _$ContinueIssuanceResult {
  const factory ContinueIssuanceResult.previews(
    List<WalletCard> previews,
  ) = ContinueIssuancePreviews;

  /// The issuer does not provide previews, so the cards can only be shown once they are issued.
  const factory ContinueIssuanceResult.noPreviews({
    required Organization organization,
  }) = ContinueIssuanceNoPreviews;
}
