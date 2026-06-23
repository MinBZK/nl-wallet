import 'package:freezed_annotation/freezed_annotation.dart';

import '../attribute/attribute.dart';
import '../organization.dart';
import '../policy/policy.dart';
import 'disclose_card_request.dart';
import 'disclosure_session_type.dart';
import 'disclosure_type.dart';

part 'start_disclosure_result.freezed.dart';

@freezed
sealed class StartDisclosureResult with _$StartDisclosureResult {
  const factory StartDisclosureResult.readyToDisclose({
    required Organization relyingParty,
    required String originUrl,
    required LocalizedText requestPurpose,
    required bool sharedDataWithOrganizationBefore,
    required DisclosureSessionType sessionType,
    required List<DiscloseCardRequest> cardRequests,
    required Policy policy,
    required DisclosureType type,
  }) = StartDisclosureReadyToDisclose;

  const factory StartDisclosureResult.missingAttributes({
    required Organization relyingParty,
    required String originUrl,
    required LocalizedText requestPurpose,
    required bool sharedDataWithOrganizationBefore,
    required DisclosureSessionType sessionType,
    required List<MissingAttribute> missingAttributes,
  }) = StartDisclosureMissingAttributes;
}
