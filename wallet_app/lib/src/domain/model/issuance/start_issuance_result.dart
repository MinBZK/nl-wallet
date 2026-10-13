import 'package:collection/collection.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import '../attribute/attribute.dart';
import '../card/wallet_card.dart';
import '../disclosure/disclose_card_request.dart';
import '../disclosure/disclosure_session_type.dart';
import '../organization.dart';
import '../policy/policy.dart';

part 'start_issuance_result.freezed.dart';

@freezed
sealed class StartIssuanceResult with _$StartIssuanceResult {
  const StartIssuanceResult._();

  const factory StartIssuanceResult.authorizationRequired(
    String authUrl,
  ) = StartIssuanceAuthorizationRequired;

  const factory StartIssuanceResult.preAuthorizedOffer(
    List<WalletCard> previews,
  ) = StartIssuancePreAuthorizedOffer;

  /// The issuer does not provide previews, so the cards can only be shown once they are issued.
  const factory StartIssuanceResult.noPreviews({
    required Organization organization,
  }) = StartIssuanceNoPreviews;

  const factory StartIssuanceResult.readyToDisclose({
    required Organization relyingParty,
    required LocalizedText requestPurpose,
    required DisclosureSessionType sessionType,
    required List<DiscloseCardRequest> cardRequests,
    required Policy policy,
  }) = StartIssuanceReadyToDisclose;

  const factory StartIssuanceResult.missingAttributes({
    required Organization relyingParty,
    required DisclosureSessionType sessionType,
    required List<MissingAttribute> missingAttributes,
  }) = StartIssuanceMissingAttributes;

  Organization? get relyingParty => map(
    authorizationRequired: (_) => null,
    preAuthorizedOffer: (value) => value.previews.firstOrNull?.issuer,
    noPreviews: (value) => value.organization,
    readyToDisclose: (value) => value.relyingParty,
    missingAttributes: (value) => value.relyingParty,
  );

  DisclosureSessionType? get sessionType => map(
    authorizationRequired: (_) => null,
    preAuthorizedOffer: (_) => null,
    noPreviews: (_) => null,
    readyToDisclose: (value) => value.sessionType,
    missingAttributes: (value) => value.sessionType,
  );
}
