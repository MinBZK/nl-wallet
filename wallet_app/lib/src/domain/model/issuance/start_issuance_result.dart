import 'package:collection/collection.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import '../attribute/attribute.dart';
import '../card/wallet_card.dart';
import '../disclosure/disclose_card_request.dart';
import '../disclosure/disclosure_session_type.dart';
import '../disclosure/disclosure_type.dart';
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

  const factory StartIssuanceResult.readyToDisclose({
    required Organization relyingParty,
    required String originUrl,
    required LocalizedText requestPurpose,
    required DisclosureSessionType sessionType,
    required DisclosureType type,
    required List<DiscloseCardRequest> cardRequests,
    required Policy policy,
    required bool sharedDataWithOrganizationBefore,
  }) = StartIssuanceReadyToDisclose;

  const factory StartIssuanceResult.missingAttributes({
    required Organization relyingParty,
    required String originUrl,
    required LocalizedText requestPurpose,
    required DisclosureSessionType sessionType,
    required List<MissingAttribute> missingAttributes,
    required bool sharedDataWithOrganizationBefore,
  }) = StartIssuanceMissingAttributes;

  Organization? get relyingParty => map(
    authorizationRequired: (_) => null,
    preAuthorizedOffer: (value) => value.previews.firstOrNull?.issuer,
    readyToDisclose: (value) => value.relyingParty,
    missingAttributes: (value) => value.relyingParty,
  );

  DisclosureSessionType? get sessionType => map(
    authorizationRequired: (_) => null,
    preAuthorizedOffer: (_) => null,
    readyToDisclose: (value) => value.sessionType,
    missingAttributes: (value) => value.sessionType,
  );
}
